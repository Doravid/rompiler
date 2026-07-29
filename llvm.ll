; ModuleID = 'main'
source_filename = "main"

define i64 @main() {
entry:
  %x = alloca i64, align 8
  store i64 5, ptr %x, align 4
  %p = alloca ptr, align 8
  store ptr %x, ptr %p, align 8
  %p1 = load ptr, ptr %p, align 8
  %deref = load i64, ptr %p1, align 4
  ret i64 %deref
}