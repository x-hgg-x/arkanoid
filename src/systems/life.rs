use crate::resources::{Game, GameEvent};
use crate::states::LIFE_TEXT_ID;
use crate::systems::LifeEvent;

use amethyst::{
    derive::SystemDesc,
    ecs::{Read, System, SystemData as _, World, Write, WriteStorage},
    prelude::*,
    shrev::{EventChannel, ReaderId},
    ui::{UiFinder, UiText},
};

#[derive(SystemDesc)]
pub struct LifeSystem {
    reader: ReaderId<LifeEvent>,
}

impl LifeSystem {
    pub fn new(world: &mut World) -> Self {
        <Self as System>::SystemData::setup(world);
        Self {
            reader: world.write_resource::<EventChannel<LifeEvent>>().register_reader(),
        }
    }
}

type SystemData<'s> = (Write<'s, Game>, WriteStorage<'s, UiText>, UiFinder<'s>, Read<'s, EventChannel<LifeEvent>>);

impl<'s> System<'s> for LifeSystem {
    type SystemData = SystemData<'s>;

    fn run(&mut self, (mut game, mut ui_texts, ui_finder, life_event_channel): SystemData) {
        for _event in life_event_channel.read(&mut self.reader) {
            game.lifes -= 1;

            if let Some(ui_text) = ui_finder.find(LIFE_TEXT_ID).and_then(|entity| ui_texts.get_mut(entity)) {
                ui_text.text = format!("LIFES: {}", game.lifes);
            }

            if game.lifes <= 0 {
                game.event = Some(GameEvent::GameOver);
            }
        }
    }
}
