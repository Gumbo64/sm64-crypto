use std::os::raw::{c_void, c_uchar, c_int, c_short, c_ushort, c_uint, c_float, c_char};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]

pub struct MarioState {
    pub playerIndex: c_ushort,
    pub input: c_ushort,

    pub flags: c_uint,
    pub particleFlags: c_uint,
    pub action: c_uint,
    pub prevAction: c_uint,
    pub terrainSoundAddend: c_uint,

    pub actionState: c_ushort,
    pub actionTimer: c_ushort,

    pub actionArg: c_uint,
    pub intendedMag: c_float,
    pub intendedYaw: c_short,
    pub invincTimer: c_short,
    pub framesSinceA: c_uchar,
    pub framesSinceB: c_uchar,
    pub wallKickTimer: c_uchar,
    pub doubleJumpTimer: c_uchar,
    pub faceAngle: [c_short; 3],
    pub angleVel: [c_short; 3],
    pub slideYaw: c_short,
    pub twirlYaw: c_short,
    pub pos: [c_float; 3],
    pub vel: [c_float; 3],
    pub forwardVel: c_float,
    pub slideVelX: c_float,
    pub slideVelZ: c_float,
    pub wall: *mut c_void,
    pub ceil: *mut c_void,
    pub floor: *mut c_void,
    pub ceilHeight: c_float,
    pub floorHeight: c_float,
    pub floorAngle: c_short,
    pub waterLevel: c_short,
    pub interactObj: *mut c_void,
    pub heldObj: *mut c_void,
    pub usedObj: *mut c_void,
    pub riddenObj: *mut c_void,
    pub marioObj: *mut c_void,
    pub spawnInfo: *mut c_void,
    pub area: *mut c_void,
    pub statusForCamera: *mut c_void,
    pub marioBodyState: *mut c_void,
    pub controller: *mut c_void,
    pub animation: *mut c_void,
    pub collidedObjInteractTypes: c_uint,
    pub numCoins: c_short,
    pub numStars: c_short,
    pub numKeys: c_char,
    pub numLives: c_char,
    pub health: c_short,
    pub unkB0: c_short,
    pub hurtCounter: c_uchar,
    pub healCounter: c_uchar,
    pub squishTimer: c_uchar,
    pub fadeWarpOpacity: c_uchar,
    pub capTimer: c_ushort,
    pub prevNumStarsForDialog: c_short,
    pub peakHeight: c_float,
    pub quicksandDepth: c_float,
    pub unkC4: c_float,
    pub currentRoom: c_short,
    pub heldByObj: *mut c_void,
    pub isSnoring: c_uchar,
    pub bubbleObj: *mut c_void,
    pub freeze: c_uchar,
    pub splineKeyframe: *mut c_void,
    pub splineKeyframeFraction: c_float,
    pub splineState: c_int,
    pub nonInstantWarpPos: [c_float; 3],
    pub character: *mut c_void,
    pub wasNetworkVisible: c_uchar,
    pub minimumBoneY: c_float,
    pub curAnimOffset: c_float,
    pub knockbackTimer: c_uchar,
    pub specialTripleJump: c_uchar,
    pub wallNormal: [c_float; 3],
    pub visibleToEnemies: c_uchar,
    pub cap: c_uint,
    pub bounceSquishTimer: c_uchar,
    pub skipWarpInteractionsTimer: c_uchar,
    pub dialogId: c_short,
}

// #[repr(C)]
// #[derive(Debug, Copy, Clone)]
// #[allow(non_snake_case)]
// pub struct Pixels {
//     pub pixelsWidth: c_int,
//     pub pixelsHeight: c_int,
//     pub pixels: *mut c_uchar,
// }

#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
pub struct GameInfo {
    pub inCredits: bool,
    pub courseNum: c_short,
    pub actNum: c_short,
    pub areaIndex: c_short,
}
