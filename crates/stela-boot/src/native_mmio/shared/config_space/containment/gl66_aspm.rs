//! VetTV GL66 experimental (`--gl66-aspm-off`): PCIe ASPM turned off on a
//! cold requester's link, at the requester and at the forwarding bridge
//! directly above it, before its driver starts it.
use super::*;
use stela_drivers::{
    block::BlockError,
    bus::pci::{Function, LinkControlChange, disable_aspm_device_and_port, link_control_word_of},
};

/// The requester's Link Control before and after, and the bridge's with its
/// address when one sits above it.
pub(crate) type AspmOff = (LinkControlChange, Option<(Address, LinkControlChange)>);

impl Source {
    /// The Link Control word of the original forwarding bridge directly
    /// above `address`, with the bridge's address; `None` for a function on
    /// a root bus. Reads only.
    pub(crate) fn port_link_control(
        &self,
        address: Address,
    ) -> Result<Option<(Address, u16)>, BlockError> {
        let cohort = self
            .pci_slots
            .0
            .try_borrow()
            .map_err(|_| BlockError::Unsupported("the PCI fabric is busy"))?;
        let Cohort::Parked(fabric) = &*cohort else {
            return Err(BlockError::Unsupported("the PCI fabric is not contained"));
        };
        let at = fabric
            .slots
            .binary_search_by_key(&address, |slot| slot.address)
            .map_err(|_| BlockError::Unsupported("the PCI function is outside the fabric"))?;
        let topology::Parent::Bridge(bridge) = fabric.slots[at].route else {
            return Ok(None);
        };
        let parent = fabric
            .slots
            .binary_search_by_key(&bridge, |slot| slot.address)
            .map_err(|_| BlockError::Unsupported("the bridge above is outside the fabric"))?;
        let Some(Forwarding::Open { config, .. }) = &fabric.slots[parent].forwarding else {
            return Err(BlockError::Unsupported("the bridge above does not forward"));
        };
        let word = link_control_word_of(config)?.ok_or(BlockError::Unsupported(
            "the bridge above has no PCIe link control",
        ))?;
        Ok(Some((bridge, word)))
    }

    /// Clear ASPM L0s and L1 at the cold requester `function` and at the
    /// original forwarding bridge above it
    /// ([`disable_aspm_device_and_port`]). A requester on a root bus has
    /// only its own word changed. The bridge's forwarding is rechecked after.
    /// Only the two Link Control words are written; no reset, routing or
    /// command change is made.
    pub(crate) fn disable_link_aspm(
        &self,
        function: &mut Function<OwnedConfig>,
    ) -> Result<AspmOff, BlockError> {
        let address = function.address;
        let refused = |why: &'static str| {
            crate::output::arguments(format_args!(
                "HOST PCI {address} ASPM change refused: {why}\n"
            ));
            BlockError::Unsupported(why)
        };
        if function.config().address() != address {
            return Err(refused("the PCI function's owner does not match"));
        }
        let mut cohort = self
            .pci_slots
            .0
            .try_borrow_mut()
            .map_err(|_| refused("the PCI fabric is busy"))?;
        let Cohort::Parked(fabric) = &mut *cohort else {
            return Err(refused("the PCI fabric is not contained"));
        };
        let at = fabric
            .slots
            .binary_search_by_key(&address, |slot| slot.address)
            .map_err(|_| refused("the PCI function is outside the contained fabric"))?;
        let topology::Parent::Bridge(bridge) = fabric.slots[at].route else {
            let (own, _) = disable_aspm_device_and_port::<_, OwnedConfig>(function, None)?;
            return Ok((own, None));
        };
        let parent = fabric
            .slots
            .binary_search_by_key(&bridge, |slot| slot.address)
            .map_err(|_| refused("the bridge above the PCI function is outside the fabric"))?;
        let Some(Forwarding::Open { config, .. }) = &mut fabric.slots[parent].forwarding else {
            return Err(refused(
                "the bridge above the PCI function does not forward",
            ));
        };
        let changed = disable_aspm_device_and_port(function, Some(config))?;
        if let Err(error) = fabric.slots[parent]
            .forwarding
            .as_ref()
            .expect("checked forwarding")
            .check()
        {
            crate::output::arguments(format_args!(
                "HOST PCI {address} ASPM change: the bridge's forwarding changed: {error}\n"
            ));
            return Err(BlockError::Unsupported("the bridge's forwarding changed"));
        }
        let (own, port) = changed;
        Ok((own, port.map(|port| (bridge, port))))
    }
}
