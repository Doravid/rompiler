; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
entry:
  %arr = alloca [5 x i64], align 8
  %arr_idx_ptr = getelementptr [5 x i64], ptr %arr, i64 0, i64 2
  store i64 10, ptr %arr_idx_ptr, align 4
  %arr_idx_ptr1 = getelementptr [5 x i64], ptr %arr, i64 0, i64 2
  %elem_val = load i64, ptr %arr_idx_ptr1, align 4
  ret i64 %elem_val
}