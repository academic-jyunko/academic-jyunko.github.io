---
layout: post
title: "读《Attention Is All You Need》"
author: 简律纯
date: 2023-06-11
mathjax: true
summary: |
  无事可做的一天，于是想起来之前自己编译的那篇"Attention Is All You Need"，我打印出来后便是再也没看过了，于是把它翻了个底朝天。在这里记下我阅读它时所做的笔记。
---

<p align="center">Part 1</p>

> _Abstract_  

The dominant sequence transduction models are based on complex recurrent or convolutional neural networks that include an encoder and a decoder. The best performing models also connect the encoder and decoder through an attention mechanism. We propose a new simple network architecture, the Transformer. based solely on attention mechanisms, dispensing with recurrence and convolutions entirely. Experiments on two machine translation tasks show these models to be superior in quality while being more parallelizable and requiring significantly less time to train. Our model achieves 28.4 BLEU on the WMT 2014 English to-German translation task, improving over the existing best results, including ensembles, by over 2 BLEU. On the WMT 2014 English-to-French translation task our model establishes a new single-model state-of-the-art BLEU score of41.8 after training for 3.5 days on eight GPUs, a small fraction of the training costs of the best models from the literature. We show that the Transformer generalizes well to other tasks by applying it successfully to English constituency parsing both with large and limited training data.

摘要中表示这篇论文介绍了一种新型的神经网络模型架构：`Transformer`，它完全基于注意力机制，在不使用循环和卷积的前提下实现序列转换任务。该模型在两个机器翻译任务中取得了表现优异的结果，训练时间缩短，且泛化性能良好。注意力机制是一种可以让神经网络更关注与输出结果相关的输入序列部分的技术。

> _Q&A_

<details close>
<summary>
1. 什么是注意力机制?
</summary>
<q>
注意力机制是一种常用于神经网络模型中的技术，可以让模型在处理输入数据时更加关注与输出结果相关的部分。通俗地说，就是让模型能够将更多的注意力集中在对预测结果有贡献的输入序列部分。在序列转换任务中，注意力机制常被用于连接编码器和解码器，使得模型能够根据解码器当前的输出状态，在编码器的输入序列中寻找最相关的信息进行输出预测。相比于传统的固定权重分配方式，注意力机制能够在不同时间步根据解码器的输出状态自适应地调整权重，从而提高模型的性能表现。
</q>
</details>

***