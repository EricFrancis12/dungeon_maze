use bevy::{
    asset::{AssetPath, LoadState},
    prelude::*,
};
use std::{thread::sleep, time::Duration};

pub fn blocking_load<'a, A>(
    asset_server: &Res<AssetServer>,
    path: impl Into<AssetPath<'a>>,
    millis: u64,
) -> Handle<A>
where
    A: Asset,
{
    let handle: Handle<A> = asset_server.load(path.into());
    while asset_server.get_load_state(&handle) != Some(LoadState::Loaded) {
        sleep(Duration::from_millis(millis));
    }
    handle
}
