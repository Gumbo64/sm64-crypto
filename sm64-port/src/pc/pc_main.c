#include <stdint.h>
#include <stdlib.h>
#include <sys/types.h>

#if defined(TARGET_WEB) && !defined(HEADLESS_VERSION)
#include <emscripten.h>
#include <emscripten/html5.h>
#endif

#include "sm64.h"

#include "game/memory.h"
#include "audio/external.h"

#include "gfx/gfx_pc.h"
#include "gfx/gfx_opengl.h"
#include "gfx/gfx_direct3d11.h"
#include "gfx/gfx_direct3d12.h"
#include "gfx/gfx_dxgi.h"
#include "gfx/gfx_glx.h"
#include "gfx/gfx_sdl.h"
#include "gfx/gfx_dummy.h"

#include "audio/audio_api.h"
#include "audio/audio_wasapi.h"
#include "audio/audio_pulse.h"
#include "audio/audio_alsa.h"
#include "audio/audio_sdl.h"
#include "audio/audio_null.h"

#include "controller/controller_keyboard.h"
#include "controller/controller_recorded_tas.h"

#include "configfile.h"

#include "compat.h"

#define CONFIG_FILE "sm64config.txt"

OSMesg gMainReceivedMesg;
OSMesgQueue gSIEventMesgQueue;

s8 gResetTimer;
s8 gNmiResetBarsTimer;
s8 gDebugLevelSelect;
s8 gShowProfiler;
s8 gShowDebugText;

static struct AudioAPI *audio_api;
static struct GfxWindowManagerAPI *wm_api;
static struct GfxRenderingAPI *rendering_api;

extern void gfx_run(Gfx *commands);
extern void thread5_game_loop(void *arg);
extern void create_next_audio_buffer(s16 *samples, u32 num_samples);
void game_loop_one_iteration(void);

void dispatch_audio_sptask(UNUSED struct SPTask *spTask) {
}

void set_vblank_handler(UNUSED s32 index, UNUSED struct VblankHandler *handler, UNUSED OSMesgQueue *queue, UNUSED OSMesg *msg) {
}


static uint8_t inited = 0;

#include "game/game_init.h" // for gGlobalTimer
void exec_display_list(struct SPTask *spTask) {
    if (!inited) {
        return;
    }
    gfx_run((Gfx *)spTask->task.t.data_ptr);
}

// #define printf

#ifdef VERSION_EU
#define SAMPLES_HIGH 656
#define SAMPLES_LOW 640
#else
#define SAMPLES_HIGH 544
#define SAMPLES_LOW 528
#endif

static int audio_enabled = 1;

void produce_one_frame(void) {
    gfx_start_frame();
    game_loop_one_iteration();
    
    if (audio_enabled) {
        int samples_left = audio_api->buffered();
        u32 num_audio_samples = samples_left < audio_api->get_desired_buffered() ? SAMPLES_HIGH : SAMPLES_LOW;
        //printf("Audio samples: %d %u\n", samples_left, num_audio_samples);
        s16 audio_buffer[SAMPLES_HIGH * 2 * 2];
        for (int i = 0; i < 2; i++) {
            /*if (audio_cnt-- == 0) {
                audio_cnt = 2;
            }
            u32 num_audio_samples = audio_cnt < 2 ? 528 : 544;*/
            create_next_audio_buffer(audio_buffer + i * (num_audio_samples * 2), num_audio_samples);
        }
        // printf("Audio samples before submitting: %d\n", audio_api->buffered());
        audio_api->play((u8 *)audio_buffer, 2 * num_audio_samples * 4);
    }

    gfx_end_frame();
}

static void save_config(void) {
    configfile_save(CONFIG_FILE);
}

static void on_fullscreen_changed(bool is_now_fullscreen) {
    configFullscreen = is_now_fullscreen;
}

void nothing() {}

#include <stdio.h>
void main_func() {
#ifdef USE_SYSTEM_MALLOC
    main_pool_init();
    gGfxAllocOnlyPool = alloc_only_pool_init();
#else
    static u64 pool[0x165000/8 / 4 * sizeof(void *)];
    main_pool_init(pool, pool + sizeof(pool) / sizeof(pool[0]));
#endif
    gEffectsMemoryPool = mem_pool_init(0x4000, MEMORY_POOL_LEFT);

    // configfile_load(CONFIG_FILE);
    // atexit(save_config);
#if defined(TARGET_WEB) && !defined(HEADLESS_VERSION)
    emscripten_set_main_loop(nothing, 0, 0);
#endif
#if defined(HEADLESS_VERSION)
    rendering_api = &gfx_dummy_renderer_api;
    wm_api = &gfx_dummy_wm_api;
#else
#if defined(ENABLE_DX12)
    rendering_api = &gfx_direct3d12_api;
    wm_api = &gfx_dxgi_api;
#elif defined(ENABLE_DX11)
    rendering_api = &gfx_direct3d11_api;
    wm_api = &gfx_dxgi_api;
#elif defined(ENABLE_OPENGL)
    rendering_api = &gfx_opengl_api;
    #if defined(__linux__) || defined(__BSD__)
        wm_api = &gfx_glx;
    #else
        wm_api = &gfx_sdl;
    #endif
#endif
#endif
    gfx_init(wm_api, rendering_api, "Super Mario 64 PC-Port", configFullscreen, 1);
    
    wm_api->set_fullscreen_changed_callback(on_fullscreen_changed);
    wm_api->set_keyboard_callbacks(keyboard_on_key_down, keyboard_on_key_up, keyboard_on_all_keys_up);

#if defined(HEADLESS_VERSION)
    rendering_api = &gfx_dummy_renderer_api;
    wm_api = &gfx_dummy_wm_api;
#else
#if HAVE_WASAPI
    if (audio_api == NULL && audio_wasapi.init()) {
        audio_api = &audio_wasapi;
    }
#endif
#if HAVE_PULSE_AUDIO
    if (audio_api == NULL && audio_pulse.init()) {
        audio_api = &audio_pulse;
    }
#endif
#if HAVE_ALSA
    if (audio_api == NULL && audio_alsa.init()) {
        audio_api = &audio_alsa;
    }
#endif
#ifdef TARGET_WEB
    if (audio_api == NULL && audio_sdl.init()) {
        audio_api = &audio_sdl;
    }
#endif
#endif
    if (audio_api == NULL) {
        audio_api = &audio_null;
    }

    audio_init();
    sound_init();

    thread5_game_loop(NULL);

    inited = 1;
}

int main() {
    main_func();
}


// EXPORTED FUNCTIONS

// main_func
// 

#include "game/level_update.h"
#include "game/camera.h"

#include "random_action.h"
#include "controller/controller_recorded_tas.h"

void update_seed() {
    rng_update((uint32_t)gMarioState->pos[0]);
    rng_update((uint32_t)gMarioState->pos[1]);
    rng_update((uint32_t)gMarioState->pos[2]);
    rng_update((uint32_t)gMarioState->vel[0]);
    rng_update((uint32_t)gMarioState->vel[1]);
    rng_update((uint32_t)gMarioState->vel[2]);
}

// #include "controller/controller.h"

// wasmtime rust only allows 32 and 64 for some reason.
void step_game(int button, int stick_x, int stick_y) {
    set_tas_controller(button, stick_x, stick_y);
    produce_one_frame();
    update_seed();
}

void set_audio_enabled(int t) {
    audio_enabled = t;
}

struct MarioState *get_mario_state() {
    return gMarioState;
}

// just make them all s32, something wrong with reading otherwise
struct GameState {
    s32 numStars;
    Vec3f pos;
    Vec3f vel;
    Vec3f lakituPos;
    s32 lakituYaw;
    s32 inCredits;
    s32 courseNum;
    s32 actNum;
    s32 areaIndex;
};

struct GameState *get_game_state() {
    static struct GameState info = {0};
    info.numStars = gMarioState->numStars;
    info.inCredits = gCurrCreditsEntry != NULL;
    info.courseNum = gCurrCourseNum;
    info.actNum = gCurrActNum;
    info.areaIndex = gCurrAreaIndex;
    info.lakituYaw = gLakituState.yaw;

    for (int i = 0; i < 3; i++) {
        info.pos[i] = gMarioState->pos[i];
        info.vel[i] = gMarioState->vel[i];
        info.lakituPos[i] = gLakituState.pos[i];
    }
    return &info;
}