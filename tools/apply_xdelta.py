"""Applies an xdelta3 patch, so a romhack that ships as one can be built without running its
patcher. A minimal VCDIFF (RFC 3284) decoder: xdelta3's application header and Adler-32 window
checksums are handled; secondary compression (`-S`) and target-copy windows are not.

    python tools/apply_xdelta.py <source.iso> <patch.xdelta> <target.iso>

Every window's checksum is checked, so a wrong source ISO stops with an error rather than
writing a scrambled image. Then extract the files with `tools/extract_gcm.py`.
"""
import mmap
import sys
import zlib


def build_table():
    # RFC 3284 section 5.6 default code table: (type1,size1,mode1,type2,size2,mode2)
    NOOP, ADD, RUN, COPY = 0, 1, 2, 3
    t = [(RUN, 0, 0, NOOP, 0, 0)]
    for s in [0] + list(range(1, 18)):
        t.append((ADD, s, 0, NOOP, 0, 0))
    for m in range(9):
        t.append((COPY, 0, m, NOOP, 0, 0))
        for s in range(4, 19):
            t.append((COPY, s, m, NOOP, 0, 0))
    for m in range(6):
        for a in range(1, 5):
            for c in range(4, 7):
                t.append((ADD, a, 0, COPY, c, m))
    for m in range(6, 9):
        for a in range(1, 5):
            t.append((ADD, a, 0, COPY, 4, m))
    for m in range(9):
        t.append((COPY, 4, m, ADD, 1, 0))
    assert len(t) == 256, len(t)
    return t


TABLE = build_table()


class Reader:
    def __init__(self, buf, pos=0):
        self.b, self.p = buf, pos

    def byte(self):
        v = self.b[self.p]
        self.p += 1
        return v

    def varint(self):
        v = 0
        while True:
            c = self.b[self.p]
            self.p += 1
            v = (v << 7) | (c & 0x7F)
            if not c & 0x80:
                return v

    def take(self, n):
        v = self.b[self.p:self.p + n]
        self.p += n
        return v


def main(src_path, patch_path, out_path):
    src = open(src_path, "rb")
    smap = mmap.mmap(src.fileno(), 0, access=mmap.ACCESS_READ)
    pf = open(patch_path, "rb")
    pm = mmap.mmap(pf.fileno(), 0, access=mmap.ACCESS_READ)
    r = Reader(pm)
    assert bytes(r.take(3)) == b"\xd6\xc3\xc4" and r.byte() == 0
    hdr = r.byte()
    assert hdr & 0x03 == 0, f"unsupported header indicator {hdr:#x}"
    if hdr & 0x04:  # xdelta3 application header (file names); skip it
        r.take(r.varint())
    out = open(out_path, "wb")
    windows = 0
    total = 0
    while r.p < len(pm):
        win = r.byte()
        seg = b""
        if win & 0x03:
            seg_len = r.varint()
            seg_pos = r.varint()
            assert not win & 0x02, "VCD_TARGET windows not supported"
            seg = smap[seg_pos:seg_pos + seg_len]
        r.varint()  # delta encoding length
        tlen = r.varint()
        assert r.byte() == 0, "compressed sections not supported"
        dlen, ilen, alen = r.varint(), r.varint(), r.varint()
        cksum = None
        if win & 0x04:
            cksum = int.from_bytes(r.take(4), "big")
        data = Reader(r.take(dlen))
        inst = Reader(r.take(ilen))
        addr = Reader(r.take(alen))
        near = [0, 0, 0, 0]
        ni = 0
        same = [0] * (3 * 256)
        slen = len(seg)
        t = bytearray()

        def copy(size, mode):
            nonlocal ni
            here = slen + len(t)
            if mode == 0:
                a = addr.varint()
            elif mode == 1:
                a = here - addr.varint()
            elif mode < 6:
                a = near[mode - 2] + addr.varint()
            else:
                a = same[(mode - 6) * 256 + addr.byte()]
            near[ni] = a
            ni = (ni + 1) % 4
            same[a % 768] = a
            if a + size <= slen:
                t.extend(seg[a:a + size])
            else:
                if a < slen:
                    n = slen - a
                    t.extend(seg[a:])
                    a, size = slen, size - n
                a -= slen
                while size:
                    n = min(size, len(t) - a)
                    t.extend(t[a:a + n])
                    a += n
                    size -= n

        while inst.p < len(inst.b):
            code = inst.byte()
            for typ, size, mode in (TABLE[code][0:3], TABLE[code][3:6]):
                if typ == 0:
                    continue
                if size == 0:
                    size = inst.varint()
                if typ == 1:
                    t.extend(data.take(size))
                elif typ == 2:
                    t.extend(data.take(1) * size)
                else:
                    copy(size, mode)
        assert len(t) == tlen, (len(t), tlen)
        if cksum is not None:
            assert zlib.adler32(t) == cksum, f"adler32 mismatch in window {windows}"
        out.write(t)
        windows += 1
        total += tlen
    out.close()
    print(f"{windows} windows, {total} bytes")


if __name__ == "__main__":
    main(*sys.argv[1:4])
