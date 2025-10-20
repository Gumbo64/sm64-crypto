#include <stdint.h>
#include <stdlib.h>
#include <sys/types.h>

#ifdef TARGET_WEB
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

void produce_one_frame(void) {
    gfx_start_frame();
    game_loop_one_iteration();
    
    // int samples_left = audio_api->buffered();
    // u32 num_audio_samples = samples_left < audio_api->get_desired_buffered() ? SAMPLES_HIGH : SAMPLES_LOW;
    // //printf("Audio samples: %d %u\n", samples_left, num_audio_samples);
    // s16 audio_buffer[SAMPLES_HIGH * 2 * 2];
    // for (int i = 0; i < 2; i++) {
    //     /*if (audio_cnt-- == 0) {
    //         audio_cnt = 2;
    //     }
    //     u32 num_audio_samples = audio_cnt < 2 ? 528 : 544;*/
    //     create_next_audio_buffer(audio_buffer + i * (num_audio_samples * 2), num_audio_samples);
    // }
    // printf("Audio samples before submitting: %d\n", audio_api->buffered());
    // audio_api->play((u8 *)audio_buffer, 2 * num_audio_samples * 4);
    
    gfx_end_frame();
}
#include "game/level_update.h"
#include "game/camera.h"
bool has_won() {
    return gMarioState->numStars > 0;
}
#include "controller/controller_recorded_tas.h"

#ifndef TARGET_WEB
#include <time.h>
#endif
#include <string.h>


unsigned long simple_hash(const char *str) {
    unsigned long hash = 5381;
    int c;

    while ((c = *str++)) {
        hash = ((hash << 5) + hash) + c; // hash * 33 + c
    }
    
    return hash;
}

void update_seed() {
    rng_update(rng_next() ^ (uint32_t)gMarioState->pos[0]);
    rng_update(rng_next() ^ (uint32_t)gMarioState->pos[1]);
    rng_update(rng_next() ^ (uint32_t)gMarioState->pos[2]);
    rng_update(rng_next() ^ (uint32_t)gMarioState->vel[0]);
    rng_update(rng_next() ^ (uint32_t)gMarioState->vel[1]);
    rng_update(rng_next() ^ (uint32_t)gMarioState->vel[2]);
    
    rng_update(rng_next());
}

void main_loop() {
    // #ifndef TARGET_WEB
    struct timespec ts;
    ts.tv_sec = 0;
    ts.tv_nsec = 33333333 / get_speed();
    nanosleep(&ts, &ts);
    // #endif

    update_seed();

    produce_one_frame();
    if (has_won()) {
        exit_game(0);
    }
}

#ifdef TARGET_WEB
static void em_main_loop(void) {
}

static void request_anim_frame(void (*func)(double time)) {
    EM_ASM(requestAnimationFrame(function(time) {
        dynCall("vd", $0, [time]);
    }), func);
}

static void on_anim_frame(double time) {
    static double target_time;

    time *= 0.03 / get_speed(); // milliseconds to frame count (33.333 ms -> 1)

    if (time >= target_time + 10.0) {
        // We are lagging 10 frames behind, probably due to coming back after inactivity,
        // so reset, with a small margin to avoid potential jitter later.
        target_time = time - 0.010;
    }

    for (int i = 0; i < 2; i++) {
        // If refresh rate is 15 Hz or something we might need to generate two frames
        if (time >= target_time) {
            main_loop();
            target_time = target_time + 1.0;
        }
    }

    request_anim_frame(on_anim_frame);
}
#endif

static void save_config(void) {
    configfile_save(CONFIG_FILE);
}

static void on_fullscreen_changed(bool is_now_fullscreen) {
    configFullscreen = is_now_fullscreen;
}

#include <stdio.h>
void main_func(uint32_t seed, char filename[FILENAME_MAX], int record_mode, int start_in_foreground) {
#ifdef USE_SYSTEM_MALLOC
    main_pool_init();
    gGfxAllocOnlyPool = alloc_only_pool_init();
#else
    static u64 pool[0x165000/8 / 4 * sizeof(void *)];
    main_pool_init(pool, pool + sizeof(pool) / sizeof(pool[0]));
#endif
    gEffectsMemoryPool = mem_pool_init(0x4000, MEMORY_POOL_LEFT);

    configfile_load(CONFIG_FILE);
    atexit(save_config);

#ifdef TARGET_WEB
    // emscripten_set_main_loop(em_main_loop, 0, 0);
    // request_anim_frame(on_anim_frame);
    // emscripten_cancel_main_loop();
    emscripten_set_main_loop(main_loop,0,0);
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
    gfx_init(wm_api, rendering_api, "Super Mario 64 PC-Port", configFullscreen, start_in_foreground);
    
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

    true_tas_init(filename, record_mode, seed);

#ifdef TARGET_WEB
    /*for (int i = 0; i < atoi(argv[1]); i++) {
        game_loop_one_iteration();
    }*/
    inited = 1;
#else
    inited = 1;
    while (1) {
        wm_api->main_loop(main_loop);
    }
#endif
}

// #if defined(_WIN32) || defined(_WIN64)
// #include <windows.h>
// int WINAPI WinMain(UNUSED HINSTANCE hInstance, UNUSED HINSTANCE hPrevInstance, UNUSED LPSTR pCmdLine, UNUSED int nCmdShow) {
//     main_func(1);
//     return 0;
// }
// #else
int main(int argc, char *argv[]) {
    if (argc < 4) {
        fprintf(stderr, "Usage: %s <seed> <filename> <record_mode>\n", argv[0]);
        return 1;
    }

    // Use the first command-line argument as the seed
    uint32_t seed = (uint32_t)strtoul(argv[1], NULL, 10);

    // Convert filename to char[FILENAME_MAX]
    char filename[FILENAME_MAX];
    strncpy(filename, argv[2], FILENAME_MAX);
    // filename[FILENAME_MAX - 1] = '\0'; // Ensure null termination

    // Convert record_mode from string to integer (1 or 0)
    int record_mode = atoi(argv[3]);
    if (record_mode != 0 && record_mode != 1) {
        fprintf(stderr, "Error: record_mode must be 0 or 1\n");
        return 1;
    }

    // Call main_func with the filename and record_mode
    main_func(seed, filename, record_mode, 1);

    return 0;
}
// #endif

// #include "controller/controller.h"

#include <PR/os_cont.h>

void set_controller(int stickX, int stickY, int button) {
    OSContPad *controller = &gControllerPads[0];

    controller->stick_x = stickX;
    controller->stick_y = stickY;
    // keyboard_on_key_down, keyboard_on_key_up, keyboard_on_all_keys_up
    controller->button = button;
}



void step_game(int steps, int stickX, int stickY, int button) {
    for (int i = 0; i < steps; i++) {
        set_controller(stickX, stickY, button);
        produce_one_frame();
    }
}

struct MarioState *get_mario_state() {
    return gMarioState;
}

struct GameInfo {
    bool inCredits;
    s16 courseNum;
    s16 actNum;
    s16 areaIndex;
};

struct GameInfo get_game_info() {
    struct GameInfo info = {
        .inCredits = gCurrCreditsEntry != NULL,
        .courseNum = gCurrCourseNum,
        .actNum = gCurrActNum,
        .areaIndex = gCurrAreaIndex,
    };
    return info;
}

Vec3f *get_lakitu_pos() {
    return &gLakituState.pos;
}
s16 get_lakitu_yaw() {
    return gLakituState.yaw;
}