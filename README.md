# XJTUSE 编译原理实验

## 声明

本仓库包含的实验代码完成于 2024 年 6 月 23 日，除所有 `grammar.rs` 源文件外，均为西安交通大学软件学院 2021 级某同学个人独立完成。所有 `grammar.rs` 源文件均为 [LALRPOP](https://github.com/lalrpop/lalrpop) 生成。

**对本仓库代码的二次分发、使用和修改应当遵循学术规范。**

该同学已提交的实验报告（共 3 份）SHA256 如下：

```plain
[1] 98bd6f33ba52ca9cba8f5a0cf1b7fa32e18577385c282155c6b5ac21b49cda07
[2] 60a4c1f938ed7360bee7c40c500304f0416b3dcf944b7d1a2867bff78c8fd386
[3] 8575fd9c3dbef71d3aff806763bfa16ba2214a8257b448116a6841d0ab268760
```

## 简介

本仓库对应的编译原理实验包括三个部分：词法分析器、语法分析器、基本语义分析程序。仓库中的项目整体为一个 workspace，其成员包括：

- `lexer`：对应词法分析器部分；
- `parser`：对应语法分析器部分，借助 LALRPOP 实现，实际上并不依赖 `lexer`；
- `compiler`：对应基本语义分析程序部分，借助 LALRPOP 实现，由于任务书要求不同，实现的语法与前两个部分不同；
- `syntax_util`：是用于处理语法的工具，功能包括求解 FIRST 和 FOLLOW 集合、计算 LR(1) 项目集规范族等，尚未完成，算法可能存在错误。
