## 我做了啥

### 1.  sys_write

首先观察 `user/src/bin/test1_write0.rs`。


## 1. 

- 程序陷入内核的原因有中断和异常（系统调用），请问 RISC-V 64 支持哪些中断 / 异常？

  RISC-V 一共有两大类的中断类型：局部中断（Local Interrupts）以及全局中断（Global Inerrupts）。

  局部中断是指直接与hart相连的中断，可以直接通过CSRs当中的xcause（mcause、scause、ucause）中的值得知中断的类型。在局部中断当中，只有两种标准的中断类型：计时中断（timer）以及软件中断（software）。

  全局中断实际上就是外部中断（External Interrupts）。它与PLIC相连（Platform-Level Interrupt Controller，平台级中断控制器）。实际上全局中断在多个硬件线程的情况下最为常用。PLIC用于对外部中断进行仲裁，然后再将仲裁的结果送入核内的中断控制器。

- 如何判断进入内核是由于中断还是异常？请描述陷入内核时的几个重要寄存器及其值。

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

  软件中断、 定时中断、 缺页异常等等。

## 2. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。请同学们可以自行测试这些内容 (运行 Rust 三个bad测例 ) ，描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

