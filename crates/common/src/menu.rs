use crate::inventory::equipment::EquipmentSlotName;
use bevy::prelude::{Component, States, Visibility};
use std::fmt;

#[derive(Clone, Component, Debug, Default, Eq, Hash, PartialEq)]
pub enum MenuTab {
    #[default]
    Inventory,
    Settings,
}

impl fmt::Display for MenuTab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Inventory => write!(f, "Inventory"),
            Self::Settings => write!(f, "Settings"),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub struct MenuOpen(pub bool);

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub struct ActiveMenuTab(pub MenuTab);

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Dragging {
    #[default]
    None,
    InventorySlot(usize),
    EquipmentSlot(EquipmentSlotName),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub struct DragState(pub Dragging);

#[derive(Component)]
pub struct Menu;

#[derive(Component)]
pub struct MenuContent;

#[derive(Component)]
pub struct RenderDistButton(pub u32);

#[derive(Component)]
pub struct InventorySlot(pub usize);

#[derive(Component)]
pub struct EquipmentSlot(pub EquipmentSlotName);

#[derive(Component)]
pub struct ItemImageCursorFollower;

#[derive(Component)]
pub struct VisibleOnParentHover {
    pub hovered: Visibility,
    pub not_hovered: Visibility,
}

impl Default for VisibleOnParentHover {
    fn default() -> Self {
        Self {
            hovered: Visibility::Visible,
            not_hovered: Visibility::Hidden,
        }
    }
}
