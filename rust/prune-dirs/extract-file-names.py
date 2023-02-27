import os
import shutil
import subprocess

program = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'
RUST_PATH = '/rust-own/'
SOLANA_PATH = '/solana-program-1.10.33/'
PROGRAM_PREFIX = f'{program}/code/'
dwarf = 'dwarfdump.txt'
# Created with `cat helloworld_rust_debug_optimized_dwarfump.txt | grep /home/runner > rust_core_files.txt
RUST_FILES = 'rust_core_files.txt'
# Created with `cat helloworld_rust_debug_optimized_dwarfump.txt | grep solana-program-1.10.35 > solana_sdk_files.txt`
SOLANA_FILES = 'solana_sdk_files.txt'

p0 = subprocess.Popen(
    f'cat {dwarf} | grep {RUST_PATH} > {RUST_FILES}', shell=True)
p1 = subprocess.Popen(
    f'cat {dwarf} | grep {SOLANA_PATH} > {SOLANA_FILES}', shell=True)

p0.wait()
p1.wait()


def get_file_names(dwarf_files, avoid='???'):
    files = []
    with open(dwarf_files) as f:
        for line in f:
            file = line.split('"')[1]
            if avoid in file:
                continue
            files.append(file)
    return list(set(files))


rust_files = get_file_names(RUST_FILES, '.cargo')
solana_files = get_file_names(SOLANA_FILES, '@')


def create_files(files, split_name, prefix):
    for file in files:
        print(file)
        file_rel = file.split(split_name)[1]
        folder_path = PROGRAM_PREFIX + prefix + \
            '/'.join(file_rel.split('/')[:-1])
        os.makedirs(folder_path, exist_ok=True)
        shutil.copy(file, folder_path)


create_files(rust_files, RUST_PATH + 'rust/', 'rust-solana-1.59.0/')
create_files(solana_files, SOLANA_PATH,
             'sdk' + SOLANA_PATH)
# # create_files(solana_files, 'solana-program-1.10.35/')
