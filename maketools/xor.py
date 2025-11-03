# xor.py
import sys

def xor_files(input_file, output_file, key_file):
    with open(key_file, 'rb') as kf:
        key = kf.read()
    
    with open(input_file, 'rb') as f1, open(output_file, 'wb') as f2:
        key_length = len(key)
        byte = f1.read(1)
        index = 0
        
        while byte:
            # XOR with the key in a loop
            f2.write(bytes([ord(byte) ^ key[index % key_length]]))
            byte = f1.read(1)
            index += 1

if __name__ == '__main__':
    if len(sys.argv) != 4:
        print("Usage: {} <input_file> <output_file> <key_file>".format(sys.argv[0]))
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]
    key_file = sys.argv[3]
    xor_files(input_file, output_file, key_file)
