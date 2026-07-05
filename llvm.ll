; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
entry:
  %x = alloca i64, align 8
  store i64 5, ptr %x, align 4
  %x1 = load i64, ptr %x, align 4
  %tmpadd = add i64 %x1, 10
  store i64 %tmpadd, ptr %x, align 4
  %x2 = load i64, ptr %x, align 4
  ret i64 %x2
}