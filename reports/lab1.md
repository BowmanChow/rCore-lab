# OS 实验 lab1 报告

无83

周君宝

2018011106

## 我做了啥

### 1.  sys_write

首先观察 `user/src/bin/test1_write0.rs`。 发现里面限制了函数调用栈的大小是 `0x1000`。 

然后修改 `os/src/syscall/fs.rs` 。 在进入 `sys_write` 之后判断一下地址是不是在函数调用栈之外。

因为有些数据可能在程序的 `.data` 或 `.bss` 段里， 所以在判断地址范围时应该把这个也考虑进来。

修改 `os/src/task/mod.rs` 里面的 `TaskManager` ，添加 `get_current_task()` 函数。 然后在 `os/src/syscall/fs.rs` 里 `use crate::task::TASK_MANAGER;` 。 这样在调用 `sys_write` 时可以获得当前进程的地址空间。 最后综合以上两点做判断， 就能判断程序调用的地址是否为该程序的合法地址。

### 2. sys_set_priority

修改 `os/src/task/task.rs` ， 给 `TaskControlBlock` 添加 `priority` 字段， 然后在 `sys_set_priority` 调用时判断一下 `priority` 的范围。 然后写入 `TASK_MANAGER.inner.tasks` 的 `priority` 。

### 3. stride

给 `TaskControlBlock` 添加 `stride` 字段。 修改 `TaskManager` 里面的 `run_first_task` 和 `find_next_task` 。 运行一个进程之后修改其 `stride` 。 `find_next_task` 里面搜索所有进程的 `stride` ， 选择最小的执行。

## 1. 

- 程序陷入内核的原因有中断和异常（系统调用），请问 RISC-V 64 支持哪些中断 / 异常？

  RISC-V 一共有两大类的中断类型：局部中断（Local Interrupts）以及全局中断（Global Inerrupts）。

  局部中断是指直接与hart相连的中断，可以直接通过CSRs当中的xcause（mcause、scause、ucause）中的值得知中断的类型。在局部中断当中，只有两种标准的中断类型：计时中断（timer）以及软件中断（software）。

  全局中断实际上就是外部中断（External Interrupts）。它与PLIC相连（Platform-Level Interrupt Controller，平台级中断控制器）。实际上全局中断在多个硬件线程的情况下最为常用。PLIC用于对外部中断进行仲裁，然后再将仲裁的结果送入核内的中断控制器。

- 如何判断进入内核是由于中断还是异常？请描述陷入内核时的几个重要寄存器及其值。

  中断时 `scause` 的最高有效位置 1，同步异常时置 0，

  `sepc` ： 即 Exception Program Counter，用来记录触发中断的指令的地址

  `scause` : 记录中断是否是硬件中断，以及具体的中断原因。

  `stval` ： `scause` 不足以存下中断所有的必须信息。例如缺页异常，就会将 stval 设置成需要访问但是不在内存中的地址，以便于操作系统将这个地址所在的页面加载进来。

  `stvec` ： 设置内核态中断处理流程的入口地址。存储了一个基址 BASE 和模式 MODE：

    - MODE 为 0 表示 Direct 模式，即遇到中断便跳转至 BASE 进行执行。

    - MODE 为 1 表示 Vectored 模式，此时 BASE 应当指向一个向量，存有不同处理流程的地址，遇到中断会跳转至 BASE + 4 * cause 进行处理流程。

  `sstatus` ： 具有许多状态位，控制全局中断使能等。

  `sscratch` ： 在用户态，`sscratch` 保存内核栈的地址；在内核态，`sscratch` 的值为 Trap 前用户栈的地址。

- 为了方便 os 处理， M 态软件会将 S 态异常/中断委托给 S 态软件，请指出有哪些寄存器记录了委托信息。

  `mideleg` 和 `medeleg` 这两个寄存器记录了委托信息，rustsbi中它们的值分别为 `0x222` 和 `0xb1ab` 。

- RustSBI 委托了哪些异常/中断？（提示：看看 RustSBI 在启动时输出了什么？）

  ```
  [rustsbi] mideleg: ssoft, stimer, sext (0x222)
  [rustsbi] medeleg: ima, ia, bkpt, la, sa, uecall, ipage, lpage, spage (0xb1ab)
  ```

  可以看出 RustSBI 委托了软件中断、 定时中断、 缺页异常等等。

## 2. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。请同学们可以自行测试这些内容 (运行 Rust 三个bad测例 ) ，描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

我使用的版本是 `0.2.0-alpha.4` ，没有报错， 正常退出。

## 3. 请通过 gdb 跟踪或阅读源代码了解机器从加电到跳转到 0x80200000 的过程，并描述重要的跳转。回答内核是如何进入 S 态的？

机器加电后，从0x1000开始启动，通过单步执行几步发现跳转到了0x80000000这一地址，查阅资料后得知这是qemu约定的跳转地址，也就是rustsbi的起始位置。之后查看代码可以发现执行了几步以后又跳转到了0x80002572这个地址，对照rustsbi的实现可以发现此处它引用了汇编代码，并跳转到了main这一label上，继续查看代码可以发现代码在运行了非常多条指令后调用了enter_privileged函数，其入口地址为0x80002cce（可以通过进入该函数后查看ra寄存器来定位），在该处break后单步执行可知道其跳转的地址为0x80001504，再单步执行几步后发现又跳转到了0x800023da这个地址，实际上就是通过mret切换到S模式的地址（这一点在一开始os启动后查看mepc就可以知道，前面enter_privileged函数的入口地址也是必须依靠这个跳转地址上查看ra寄存器才能反推得到的）。最后再单步执行几步后可以发现跳转到了0x80002000这一地址，也就是最终进入了os

## 4. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:

### 1. L40：刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。

`a0` 代表 `__all_traps` 压内核栈时内核栈的栈顶指针。 

两种场景： 1. `trap` 后返回用户态程序。 2. 加载用户态程序前初始化用户态程序上下文。

### 2. 这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

```
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

特殊处理了 `sstatus`、 `sepc`、 `sscratch` 三个寄存器。 把他们从栈上恢复。

意义：

  `sepc` ： 即 Exception Program Counter，用来记录触发中断的指令的地址

  `sstatus` ： 具有许多状态位，控制全局中断使能等。

  `sscratch` ： 在用户态，`sscratch` 保存内核栈的地址；在内核态，`sscratch` 的值为 Trap 前用户栈的地址。

### 3. L53-L59：为何跳过了 x2 和 x4？

```
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```


`tp(x4)` 寄存器，除非我们手动出于一些特殊用途使用它，否则一般不会被用到。不保存 `sp(x2)`，因为我们要基于它来找到每个寄存器应该被保存到的正确的位置。

### 4. L63：该指令之后，sp 和 sscratch 中的值分别有什么意义？

```
csrrw sp, sscratch, sp
```

`sp` 指向用户栈， `sscratch` 指向内核栈。


### 5. `__restore` ：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

```
sret
```

这条指令表示从 S 态返回 U 态。 执行之后会返回到 `spec` 指向的指令地址。

### 6. L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

```
csrrw sp, sscratch, sp
```

`sp` 指向内核栈， `sscratch` 

### 7. 从 U 态进入 S 态是哪一条指令发生的？

```
ecall
```


## 5. stride 算法深入

stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

- 实际情况是轮到 p1 执行吗？为什么？

不是， p2.stride 溢出了， 下一次还是 p2 执行。

我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明，在不考虑溢出的情况下, 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

- 为什么？尝试简单说明（不要求严格证明）。

一开始大家的 stride 都是 0， 一个进程（如 p1）执行后 p1.stride 就是最大的， 此时有 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。 根据归纳法， 如果任何一个状态都有 STRIDE_MAX – STRIDE_MIN <= BigStride / 2， 那么下一次执行从 stride 最小的开始执行， 执行后也必有 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

- 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if (self as * const _ as isize - other as * const _ isize).abs() <=
   			(BIG_STRIDE as isize) / 2 {
            	if self > other {
   					Some(Ordering::Greater)
   				} else {
   					Some(Ordering::Less)
   				}
   		} else {
   			if self > other {
   				Some(Ordering::Less)
   			} 
           	else {
   				Some(Ordering::Greater)
   			}
   		}
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

