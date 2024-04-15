#! /bin/sh

set -xe

LIMINE_GIT_URL="https://github.com/limine-bootloader/limine.git"

# Cargo passes the path to the built executable as the first argument.
KERNEL=$1

# Clone the `limine` repository if we don't have it yet.
if [ ! -d target/limine ]; then
    git clone $LIMINE_GIT_URL --depth=1 --branch v3.0-branch-binary target/limine
fi

# Make sure we have an up-to-date version of the bootloader.
cd target/limine
git fetch
make
cd -

# Copy the needed files into an ISO image.
mkdir -p target/iso_root
cp $KERNEL conf/limine.cfg target/limine/limine.sys target/limine/limine-cd.bin \
target/limine/limine-cd-efi.bin target/iso_root

xorriso -as mkisofs                                             \
    -b limine-cd.bin                                            \
    -no-emul-boot -boot-load-size 4 -boot-info-table            \
    --efi-boot limine-cd-efi.bin                                \
    -efi-boot-part --efi-boot-image --protective-msdos-label    \
    target/iso_root -o "$KERNEL".iso

# For the image to be bootable on BIOS systems, we must run `limine-deploy` on it.
target/limine/limine-deploy "$KERNEL".iso

# Run the created image with QEMU.
qemu-system-x86_64 \
    -machine q35 -cpu qemu64 -M smm=off\
    -D target/log.txt -d int,guest_errors\
    -serial stdio \
    -net tap,ifname=tap0,script=no,downscript=no -device rtl8139\
    "$KERNEL".iso

# Uncomment for debugging
# qemu-system-x86_64 \
#     -machine q35 -cpu qemu64 -M smm=off\
#     -S -s \
#     -D target/log.txt -d int,guest_errors\
#     -serial stdio \
#     -net tap,ifname=tap0,script=no,downscript=no -device rtl8139\
#     "$KERNEL".iso