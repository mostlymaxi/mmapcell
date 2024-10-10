# MmapCell

A common use case for `mmap` in C is to cast the mmap backed region to a struct:
```c
MyStruct* mmap_backed_mystruct;
int fd;

fd = open(path, O_RDWR | O_CREAT, 0644);
ftruncate(fd, sizeof(MyStruct));

mmap_backed_mystruct = (MyStruct*)mmap(0, sizeof(MyStruct), PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
```

## Example

This is a helpful wrapper for the same usecase:
```rust
   use mmapcell::MmapCell;

   #[repr(C)]
   struct MyStruct {
      thing1: i32,
      thing2: f64,
   }

   let cell = unsafe {
       MmapCell::<MyStruct>::new_named("/tmp/mystruct-mmap-test.bin")
   }.unwrap();

   let mmap_backed_mystruct = cell.get_mut();

   mmap_backed_mystruct.thing1 = 3;
```
