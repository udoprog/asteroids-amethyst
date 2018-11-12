use amethyst::{
    assets::{AssetStorage, Loader},
    audio::{output::Output, AudioSink, OggFormat, Source, SourceHandle},
    ecs::prelude::World,
};
use crate::resources::RandomGen;

pub struct Silent;

pub struct Sounds {
    pub pew_sfx: RandomSfx,
    pub collision_sfx: RandomSfx,
    pub explosion_sfx: RandomSfx,
}

pub struct RandomSfx {
    pub sources: Vec<SourceHandle>,
}

impl RandomSfx {
    pub fn load<'a>(world: &mut World, it: impl IntoIterator<Item = &'a str>) -> RandomSfx {
        let loader = world.read_resource::<Loader>();

        let mut sources = Vec::new();

        for p in it {
            sources.push(load_wav(&loader, &world, p));
        }

        RandomSfx {
            sources,
        }
    }

    /// Play a sound at random.
    pub fn play(&self, rand: &RandomGen, storage: &AssetStorage<Source>, output: Option<&Output>) {
        let output = match output.as_ref() {
            Some(output) => output,
            None => return,
        };

        let index = rand.next_usize() % self.sources.len();

        if let Some(sound) = self.sources.get(index).and_then(|s| storage.get(s)) {
            output.play_once(sound, 1.0);
        }
    }
}

fn load_wav(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), (), &world.read_resource())
}

#[allow(unused)]
fn load_ogg(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), (), &world.read_resource())
}

pub fn initialise_audio(world: &mut World) {
    {
        let mut sink = world.write_resource::<AudioSink>();
        sink.set_volume(0.1);
    }

    let pew_sfx = RandomSfx::load(world, vec![
        "audio/pew1.wav",
        "audio/pew2.wav",
        "audio/pew3.wav",
        "audio/pew4.wav",
        "audio/pew5.wav",
    ]);

    let collision_sfx = RandomSfx::load(world, vec![
        "audio/collision1.wav",
        "audio/collision2.wav",
        "audio/collision3.wav",
        "audio/collision4.wav",
        "audio/collision5.wav",
    ]);

    let explosion_sfx = RandomSfx::load(world, vec![
        "audio/explosion1.wav",
        "audio/explosion2.wav",
        "audio/explosion3.wav",
        "audio/explosion4.wav",
        "audio/explosion5.wav",
    ]);

    world.add_resource(Sounds {
        pew_sfx,
        collision_sfx,
        explosion_sfx,
    });

    world.add_resource(Silent);
}
