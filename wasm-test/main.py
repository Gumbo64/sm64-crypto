from wasmtime import Engine, Store, Module, Linker, WasiConfig, Memory
import random

A_BUTTON = 0x8000
B_BUTTON = 0x4000
L_TRIG = 0x0020
R_TRIG = 0x0010
Z_TRIG = 0x2000
START_BUTTON = 0x1000
U_JPAD = 0x0800
L_JPAD = 0x0200
R_JPAD = 0x0100
D_JPAD = 0x0400
U_CBUTTONS = 0x0008
L_CBUTTONS = 0x0002
R_CBUTTONS = 0x0001
D_CBUTTONS = 0x0004


import struct

from gameplay import GAMEPLAY
import numpy as np
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d import Axes3D
import imageio
import os
def create_3d_rotation_gif(positions, points_per_frame=1, filename='3d_rotation.gif'):
    frames = []
    
    for i in range(0, len(positions), points_per_frame):
        fig = plt.figure()
        ax = fig.add_subplot(111, projection='3d')

        # Plot the points for the current frame
        ax.scatter(positions[:i + points_per_frame, 0], positions[:i + points_per_frame, 2], positions[:i + points_per_frame, 1], c='b', marker='o')

        # Set limits
        ax.set_xlim([-8000, 8000])
        ax.set_ylim([-8000, 8000])
        ax.set_zlim([-8000, 8000])

        # Rotate the view
        ax.view_init(30, (i // points_per_frame) * 5 % 360)  # Adjust rotation speed and angle

        # Save the current frame to a list
        plt.savefig('temp.png')
        plt.close(fig)
        frames.append(imageio.imread('temp.png'))

    # Clean up temporary images
    os.remove('temp.png')

    # Save all frames as a GIF
    imageio.mimsave(filename, frames, fps=30)

from wasmtime import Engine, Store, Module, Linker, WasiConfig, Memory

def run():
    engine = Engine()

    # Load and compile our two modules
    linking1 = Module.from_file(engine, "./WASM/sm64_headless.us.wasm")

    # Set up our linker which is going to be linking modules together. We
    # want our linker to have wasi available, so we set that up here as well.
    linker = Linker(engine)
    linker.define_wasi()

    # Create a `Store` to hold instances, and configure wasi state
    store = Store(engine)
    wasi = WasiConfig()
    wasi.inherit_stdout()
    store.set_wasi(wasi)

    # And with that we can perform the final link and the execute the module.
    linking1 = linker.instantiate(store, linking1)
    linking1.exports(store)["main_func"](store)

    memory = linking1.exports(store)["memory"]
    step = linking1.exports(store)["step_game"]
    state = linking1.exports(store)["get_game_state"]

    def get_struct():
        addr = state(store)
        s = memory.data_ptr(store)[addr+4:addr+4+(4*3*2)]
        byte_array = bytes(s)
        float_values = [struct.unpack('f', byte_array[i:i + 4])[0] for i in range(0, len(byte_array), 4)]
        return (float_values[:3], float_values[3:])


    i = 0
    positions_list = []
    while i < len(GAMEPLAY.keys()) // 4:

        b = bytes([GAMEPLAY[str(i)] for i in range(4*i, 4*i+4)])
        # print(b)
        # stickx = random.randint(-80,80)
        # sticky = random.randint(-80,80)
        button = (b[0] << 8) | b[1]
        stickx = int(b[2])
        sticky = int(b[3])
        # button = 0
        # stickx = 0
        # sticky = 0
        # if ( (150 < i and i < 160) or (200 < i and i < 300) ):
        #     button = START_BUTTON
        
        # if (i > 300):
        #     stick_y = 80
        # if ((i % 2) == 0):
        #     button = A_BUTTON
        # if (button == START_BUTTON):
        #     print("Start")
        



        # if random.random() < 0.5:
        #     button = button | START_BUTTON
        # if random.random() < 0.5:
        #     button = button | A_BUTTON
        # if random.random() < 0.5:
        #     button = button | B_BUTTON

        step(store, button, stickx, sticky)
        pos, vel = get_struct()
        positions_list.append(pos)
        i += 1

    points = np.array(positions_list)
    create_3d_rotation_gif(points, points_per_frame=100)


if __name__ == '__main__':
    run()