cd .\bare_bones\rust_os\  ;  cargo bootimage  ;  cd ..\..\

& 'C:\Program Files\qemu\qemu-system-x86_64.exe' -drive format=raw,file=bare_bones\rust_os\target\custom_target\debug\bootimage-rust_os.bin