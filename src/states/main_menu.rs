use crate::components::PrefabHandles;
use crate::states::{GameplayState, Menu};

use amethyst::{
    ecs::Entity,
    input::{is_key_down, VirtualKeyCode},
    prelude::*,
};

#[derive(Default)]
pub struct MainMenuState {
    main_menu: Option<Entity>,
    selection: i32,
}

impl Menu for MainMenuState {
    fn get_selection(&self) -> i32 {
        self.selection
    }

    fn set_selection(&mut self, selection: i32) {
        self.selection = selection;
    }

    fn get_cursor_menu_ids(&self) -> &[&str] {
        &["cursor_new_game", "cursor_exit"]
    }
}

impl SimpleState for MainMenuState {
    fn on_start(&mut self, data: StateData<GameData>) {
        let world = data.world;

        let main_menu = world.read_resource::<PrefabHandles>().menu.main_menu.clone();
        self.main_menu = Some(world.create_entity().with(main_menu).build());
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        if let Some(entity) = self.main_menu {
            data.world.delete_entity(entity).expect("Failed to delete entity.");
        }
    }

    fn handle_event(&mut self, data: StateData<GameData>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_key_down(&event, VirtualKeyCode::Escape) || is_key_down(&event, VirtualKeyCode::Q) {
                return Trans::Quit;
            }
            if is_key_down(&event, VirtualKeyCode::Return) || is_key_down(&event, VirtualKeyCode::Space) {
                match self.selection {
                    // New game
                    0 => {
                        return Trans::Switch(Box::new(GameplayState::default()));
                    }
                    // Exit
                    1 => {
                        return Trans::Quit;
                    }
                    _ => unreachable!(),
                }
            }
            self.change_menu(data.world, &event);
        }
        Trans::None
    }
}
