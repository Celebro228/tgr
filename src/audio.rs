use quad_snd::{AudioContext, Sound};

static mut SOUND_CONTEXT: Option<AudioContext> = None;
static mut SOUNDS: Vec<Sound> = Vec::new();

pub struct Audio {
    id: usize,
}

impl Audio {
    pub fn volume(&self, volume: f32) {
        unsafe {
            SOUNDS[self.id].set_volume(&audio_ctx(), volume);
        }
    }

    pub fn play(&self) {
        unsafe {
            SOUNDS[self.id].play(&audio_ctx(), Default::default());
        }
    }

    pub fn stop(&self) {
        unsafe {
            SOUNDS[self.id].stop(&audio_ctx());
        }
    }
}

pub fn audio(path: &str) -> Audio {
    unsafe {
        if SOUND_CONTEXT.is_none() {
            SOUND_CONTEXT = Some(AudioContext::new());
        }

        let file = std::fs::read(path).unwrap();

        let audio = Sound::load(&audio_ctx(), &file);

        SOUNDS.push(audio);
        Audio {
            id: SOUNDS.len() - 1,
        }
    }
}

#[inline(always)]
fn audio_ctx() -> &'static mut AudioContext {
    unsafe { SOUND_CONTEXT.as_mut().unwrap() }
}
