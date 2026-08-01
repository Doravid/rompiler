; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
entry:
  %H = alloca i8, align 1
  store i8 72, ptr %H, align 1
  %e = alloca i8, align 1
  store i8 101, ptr %e, align 1
  %l = alloca i8, align 1
  store i8 108, ptr %l, align 1
  %o = alloca i8, align 1
  store i8 111, ptr %o, align 1
  %space = alloca i8, align 1
  store i8 32, ptr %space, align 1
  %W = alloca i8, align 1
  store i8 87, ptr %W, align 1
  %r = alloca i8, align 1
  store i8 114, ptr %r, align 1
  %d = alloca i8, align 1
  store i8 100, ptr %d, align 1
  %arr = alloca [50 x i8], align 1
  %H1 = load i8, ptr %H, align 1
  %arr_idx_ptr = getelementptr [50 x i8], ptr %arr, i64 0, i64 0
  store i8 %H1, ptr %arr_idx_ptr, align 1
  %e2 = load i8, ptr %e, align 1
  %arr_idx_ptr3 = getelementptr [50 x i8], ptr %arr, i64 0, i64 1
  store i8 %e2, ptr %arr_idx_ptr3, align 1
  %l4 = load i8, ptr %l, align 1
  %arr_idx_ptr5 = getelementptr [50 x i8], ptr %arr, i64 0, i64 2
  store i8 %l4, ptr %arr_idx_ptr5, align 1
  %l6 = load i8, ptr %l, align 1
  %arr_idx_ptr7 = getelementptr [50 x i8], ptr %arr, i64 0, i64 3
  store i8 %l6, ptr %arr_idx_ptr7, align 1
  %o8 = load i8, ptr %o, align 1
  %arr_idx_ptr9 = getelementptr [50 x i8], ptr %arr, i64 0, i64 4
  store i8 %o8, ptr %arr_idx_ptr9, align 1
  %space10 = load i8, ptr %space, align 1
  %arr_idx_ptr11 = getelementptr [50 x i8], ptr %arr, i64 0, i64 5
  store i8 %space10, ptr %arr_idx_ptr11, align 1
  %W12 = load i8, ptr %W, align 1
  %arr_idx_ptr13 = getelementptr [50 x i8], ptr %arr, i64 0, i64 6
  store i8 %W12, ptr %arr_idx_ptr13, align 1
  %o14 = load i8, ptr %o, align 1
  %arr_idx_ptr15 = getelementptr [50 x i8], ptr %arr, i64 0, i64 7
  store i8 %o14, ptr %arr_idx_ptr15, align 1
  %r16 = load i8, ptr %r, align 1
  %arr_idx_ptr17 = getelementptr [50 x i8], ptr %arr, i64 0, i64 8
  store i8 %r16, ptr %arr_idx_ptr17, align 1
  %l18 = load i8, ptr %l, align 1
  %arr_idx_ptr19 = getelementptr [50 x i8], ptr %arr, i64 0, i64 9
  store i8 %l18, ptr %arr_idx_ptr19, align 1
  %d20 = load i8, ptr %d, align 1
  %arr_idx_ptr21 = getelementptr [50 x i8], ptr %arr, i64 0, i64 10
  store i8 %d20, ptr %arr_idx_ptr21, align 1
  %arr_idx_ptr22 = getelementptr [50 x i8], ptr %arr, i64 0, i64 11
  store i8 10, ptr %arr_idx_ptr22, align 1
  %syscall_ret = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 1, i64 2, ptr %arr, i64 12)
  ret i64 %syscall_ret
}
