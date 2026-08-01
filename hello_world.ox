func main() : i64 {
    const H: i8 = 72;
    const e: i8 = 101;
    const l: i8 = 108;
    const o: i8 = 111;
    const space: i8 = 32;
    const W: i8 = 87;
    const r: i8 = 114;
    const d: i8 = 100;


    var arr: [50]u8;
    arr[0] = H;
    arr[1] = e;
    arr[2] = l;
    arr[3] = l;
    arr[4] = o;
    arr[5] = space;
    arr[6] = W;
    arr[7] = o;
    arr[8] = r;
    arr[9] = l;
    arr[10] = d;
    arr[11] = 10;
    return syscall(1, 2, &arr, 12); 
}
