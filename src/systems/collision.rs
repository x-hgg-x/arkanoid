use crate::components::{Ball, Block, Paddle, StickyBall};
use crate::states::{ARENA_HEIGHT, ARENA_WIDTH};

use amethyst::{
    core::{
        math::{Isometry2, RealField, Rotation2, Unit, Vector3},
        Time, Transform,
    },
    derive::SystemDesc,
    ecs::{Entities, Entity, Join, Read, ReadStorage, System, SystemData as _, Write, WriteStorage},
    renderer::{palette::rgb::Rgb, resources::Tint},
    shrev::EventChannel,
};

use ncollide2d::{
    query,
    shape::{Ball as BallShape, Compound, Cuboid, ShapeHandle},
};

pub struct BlockCollisionEvent {
    pub entity: Entity,
}

pub struct LifeEvent;

pub struct ScoreEvent {
    pub score: i32,
}

pub struct StopBallAttractionEvent {
    pub collision_time: f64,
}

#[derive(SystemDesc)]
pub struct CollisionSystem;

type SystemData<'s> = (
    Entities<'s>,
    WriteStorage<'s, Ball>,
    WriteStorage<'s, StickyBall>,
    WriteStorage<'s, Tint>,
    WriteStorage<'s, Transform>,
    ReadStorage<'s, Paddle>,
    ReadStorage<'s, Block>,
    Read<'s, Time>,
    Write<'s, EventChannel<BlockCollisionEvent>>,
    Write<'s, EventChannel<LifeEvent>>,
    Write<'s, EventChannel<ScoreEvent>>,
    Write<'s, EventChannel<StopBallAttractionEvent>>,
);

impl<'s> System<'s> for CollisionSystem {
    type SystemData = SystemData<'s>;

    fn run(
        &mut self,
        (
            entities,
            mut balls,
            mut sticky_balls,
            mut tints,
            mut transforms,
            paddles,
            blocks,
            time,
            mut block_collision_event_channel,
            mut life_event_channel,
            mut score_event_channel,
            mut stop_ball_attraction_event_channel,
        ): SystemData,
    ) {
        // Compute union of blocks
        let block_compound: Compound<f32> = Compound::new(
            (&blocks, &transforms)
                .join()
                .map(|(block, block_transform): (&Block, &Transform)| {
                    (
                        Isometry2::new([block_transform.translation().x, block_transform.translation().y].into(), 0.0),
                        ShapeHandle::new(Cuboid::new([block.width / 2.0, block.height / 2.0].into())),
                    )
                })
                .collect(),
        );

        // Get block entities
        let block_entities: Vec<Entity> = (&entities, &blocks).join().map(|(entity, _)| entity).collect();

        if let Some(val) = (&paddles, &transforms).join().next().map(|(paddle, paddle_transform)| (paddle, paddle_transform.translation())) {
            let (paddle, paddle_translation): (&Paddle, &Vector3<f32>) = val;
            let paddle_x = paddle_translation.x;
            let paddle_y = paddle_translation.y;

            let moving_balls: Vec<(Entity, &mut Ball, (), &mut Tint, &mut Transform)> = (&entities, &mut balls, !&sticky_balls, &mut tints, &mut transforms).join().collect();
            for (entity, ball, _, ball_tint, ball_transform) in moving_balls {
                let ball_x = ball_transform.translation().x;
                let ball_y = ball_transform.translation().y;

                // Bounce at the top, left and right of the arena
                if ball_x <= ball.radius {
                    ball.direction.as_mut_unchecked().x = ball.direction.x.abs();
                }
                if ball_x >= ARENA_WIDTH - ball.radius {
                    ball.direction.as_mut_unchecked().x = -ball.direction.x.abs();
                }
                if ball_y >= ARENA_HEIGHT - ball.radius {
                    ball.direction.as_mut_unchecked().y = -ball.direction.y.abs();
                }

                // Lose a life when ball reach the bottom of the arena
                if ball_y <= ball.radius {
                    let sticky = StickyBall {
                        width_extent: paddle.width / 2.0,
                        period: 2.0,
                    };

                    ball.velocity_mult = 1.0;
                    ball_tint.0.color = Rgb::new(1.0, 1.0, 1.0);
                    sticky_balls.insert(entity, sticky).expect("Unable to add entity to storage.");
                    ball_transform.set_translation_xyz(paddle_x, paddle.height + ball.radius, 0.0);
                    life_event_channel.single_write(LifeEvent);
                    score_event_channel.single_write(ScoreEvent { score: -1000 });
                }

                // Bounce at the paddle
                let ball_shape = BallShape::new(ball.radius);
                let ball_pos = Isometry2::new([ball_x, ball_y].into(), 0.0);

                let paddle_shape = Cuboid::new([paddle.width / 2.0, paddle.height / 2.0].into());
                let paddle_pos = Isometry2::new([paddle_x, paddle_y].into(), 0.0);

                if query::contact(&paddle_pos, &paddle_shape, &ball_pos, &ball_shape, 0.0).is_some() {
                    let angle = ((paddle_x - ball_transform.translation().x) / paddle.width * f32::pi()).min(f32::pi() / 3.0).max(-f32::pi() / 3.0);
                    ball.direction = Unit::new_unchecked([-angle.sin(), angle.cos()].into());

                    stop_ball_attraction_event_channel.single_write(StopBallAttractionEvent {
                        collision_time: time.absolute_time_seconds(),
                    });
                }

                // Bounce at the blocks
                if let Some(contact) = query::contact(&Isometry2::identity(), &block_compound, &ball_pos, &ball_shape, 0.0) {
                    stop_ball_attraction_event_channel.single_write(StopBallAttractionEvent {
                        collision_time: time.absolute_time_seconds(),
                    });

                    let angle = (-ball.direction.perp(&contact.normal)).atan2(-ball.direction.dot(&contact.normal));
                    ball.direction = -(Rotation2::new(2.0 * angle) * ball.direction);
                    ball.direction.renormalize();

                    // Get individual collided blocks
                    if block_compound.shapes().len() == block_entities.len() {
                        for (index, shape) in block_compound.shapes().iter().enumerate() {
                            let (block_isometry, block): (&Isometry2<f32>, &Cuboid<f32>) = (&shape.0, shape.1.downcast_ref().unwrap());

                            if query::contact(block_isometry, block, &ball_pos, &ball_shape, 0.0).is_some() {
                                block_collision_event_channel.single_write(BlockCollisionEvent { entity: block_entities[index] });
                                score_event_channel.single_write(ScoreEvent { score: 50 });
                            }
                        }
                    }
                }
            }
        }
    }
}
