; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
entry:
  %arr = alloca [50 x i8], align 1
  %arr_idx_ptr = getelementptr [50 x i8], ptr %arr, i64 0, i64 0
  store i8 72, ptr %arr_idx_ptr, align 1
  %arr_idx_ptr1 = getelementptr [50 x i8], ptr %arr, i64 0, i64 1
  store i8 101, ptr %arr_idx_ptr1, align 1
  %arr_idx_ptr2 = getelementptr [50 x i8], ptr %arr, i64 0, i64 2
  store i8 108, ptr %arr_idx_ptr2, align 1
  %arr_idx_ptr3 = getelementptr [50 x i8], ptr %arr, i64 0, i64 3
  store i8 108, ptr %arr_idx_ptr3, align 1
  %arr_idx_ptr4 = getelementptr [50 x i8], ptr %arr, i64 0, i64 4
  store i8 111, ptr %arr_idx_ptr4, align 1
  %arr_idx_ptr5 = getelementptr [50 x i8], ptr %arr, i64 0, i64 5
  store i8 10, ptr %arr_idx_ptr5, align 1
  %syscall_ret = call i64 asm sideeffect "syscall", "={rax},{rax},{rdi},{rsi},{rdx},~{rcx},~{r11},~{memory}"(i64 1, i64 2, ptr %arr, i64 6)
  ret i64 %syscall_ret
}
