import os
import shutil
import subprocess

#program = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'
#program = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
#program = 'CrX7kMhLC3cSsXJdT7JDgqrRVWGnUpX3gfEfxxU2NVLi'
program = 'So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo'
RUST_PATH = '/rust-own/'
RUST_VERSION = "rust-solana-1.62.0/"
SOLANA_PATH = '/solana-program-1.14.10/'
BORSH_PATH = '/borsh-0.9.3/'
PROGRAM_PREFIX = f'{program}/code/'
dwarf = 'dwarfdump.txt'
RUST_FILES = 'rust_core_files.txt'
SOLANA_FILES = 'solana_sdk_files.txt'
BORSH_FILES = 'borsh_files.txt'

p0 = subprocess.Popen(
    f'cat {dwarf} | grep {RUST_PATH} > {RUST_FILES}', shell=True)
p1 = subprocess.Popen(
    f'cat {dwarf} | grep {SOLANA_PATH} > {SOLANA_FILES}', shell=True)
p2 = subprocess.Popen(
    f'cat {dwarf} | grep {BORSH_PATH} > {BORSH_FILES}', shell=True)

p0.wait()
p1.wait()
p2.wait()

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
borsh_files = get_file_names(BORSH_FILES, '@')


def create_files(files, split_name, prefix):
    for file in files:
        print(file)
        file_rel = file.split(split_name)[1]
        folder_path = PROGRAM_PREFIX + prefix + \
            '/'.join(file_rel.split('/')[:-1])
        os.makedirs(folder_path, exist_ok=True)
        shutil.copy(file, folder_path)


create_files(rust_files, RUST_PATH + 'rust/', RUST_VERSION)
create_files(solana_files, SOLANA_PATH,
             'sdk' + SOLANA_PATH)
create_files(borsh_files, BORSH_PATH, BORSH_PATH)
