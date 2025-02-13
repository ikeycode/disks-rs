# Raw filesystem images

We provide a number of raw filesystem images (without content) to verify
the `superblock` crate, providing CI for filesystems that may change over time.

Primarily `blsforme` needs to understand the UUID for `/proc/cmdline` generation,
however extraction of volume label is also supported (`blsforme testing` in most
test images)

## btrfs.img.zst

    UUID: 829d6a03-96a5-4749-9ea2-dbb6e59368b2

## ext4.img.zst

    UUID: 731af94c-9990-4eed-944d-5d230dbe8a0d

## f2fs.img.zst

    UUID: d2c85810-4e75-4274-bc7d-a78267af7443

## luks+ext4.img.zst

    Password : abc
    Version  : LUKS2
    LUKS UUID: be373cae-2bd1-4ad5-953f-3463b2e53e59
    EXT4 UUID: e27c657e-d03c-4f89-b36d-2de6880bc2a1

## xfs.img

Limited to 12-char volume name

    UUID : 45e8a3bf-8114-400f-95b0-380d0fb7d42d
    LABEL: BLSFORME

## fat16.img.zst

  UUID : A1B2-C3D4  (volume id not a uuid)
  LABEL: TESTLABEL

  created with commands :

    dd if=/dev/zero of=fat16.img bs=512 count=32768
    mkfs.fat -F 16 -n "TESTLABEL" -i A1B2C3D4 fat16.img
    zstd fat16.img
    rm fat16.img

## fat32.img.zst

  UUID : A1B2-C3D4  (volume id not a uuid)
  LABEL: TESTLABEL

  created with commands :

    dd if=/dev/zero of=fat32.img bs=512 count=32768
    mkfs.fat -F 32 -n "TESTLABEL" -i A1B2C3D4 fat32.img
    zstd fat32.img
    rm fat32.img