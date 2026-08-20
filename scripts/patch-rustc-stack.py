"""
修改 rustc.exe 的 PE 头主线程栈大小（只改 SizeOfStackReserve/Commit 两个字段，不重建映像）
解决 Windows 编译巨型 crate 的栈溢出（主线程栈默认仅 1MB）
用法：python scripts/patch-rustc-stack.py
"""
import os
import stat
import struct
import shutil
import subprocess

path = subprocess.check_output(
    ["rustup", "which", "rustc"], text=True, encoding="utf-8"
).strip()
new_stack = 64 * 1024 * 1024  # 64MB

# 备份（已存在则跳过）
bak = path + ".bak"
if not os.path.exists(bak):
    shutil.copy2(path, bak)

# 去掉只读属性
os.chmod(path, stat.S_IWRITE | stat.S_IREAD)

with open(path, "rb") as f:
    data = bytearray(f.read())

# 定位 PE32+ Optional Header
e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]  # PE signature 偏移
opt_start = e_lfanew + 4 + 20  # 4 字节 signature + 20 字节 COFF header

magic = struct.unpack_from("<H", data, opt_start)[0]
assert magic == 0x20B, f"不是 PE32+（magic={hex(magic)}）"

old_reserve = struct.unpack_from("<Q", data, opt_start + 0x48)[0]
old_commit = struct.unpack_from("<Q", data, opt_start + 0x50)[0]

struct.pack_into("<Q", data, opt_start + 0x48, new_stack)
struct.pack_into("<Q", data, opt_start + 0x50, new_stack)

with open(path, "rb+") as f:
    f.seek(0)
    f.write(data)
    f.truncate(len(data))

print(f"旧栈: reserve={old_reserve/1024/1024:.1f}MB commit={old_commit/1024/1024:.1f}MB")
print(f"新栈: 64MB")
print(f"备份: {bak}")
