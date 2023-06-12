---
layout: post
title: "读《Attention Is All You Need》"
author: 简律纯
mathjax: true
summary: |
  无事可做的一天，于是想起来之前自己编译的那篇"Attention Is All You Need"，我打印出来后便是再也没看过了，于是把它翻了个底朝天。在这里记下我阅读它时所做的笔记。
---

<p align="center">Part 1</p>

> _Abstract_  

The dominant sequence transduction models are based on complex recurrent or convolutional neural networks that include an encoder and a decoder. The best performing models also connect the encoder and decoder through an attention mechanism. We propose a new simple network architecture, the Transformer. based solely on attention mechanisms, dispensing with recurrence and convolutions entirely. Experiments on two machine translation tasks show these models to be superior in quality while being more parallelizable and requiring significantly less time to train. Our model achieves 28.4 BLEU on the WMT 2014 English to-German translation task, improving over the existing best results, including ensembles, by over 2 BLEU. On the WMT 2014 English-to-French translation task our model establishes a new single-model state-of-the-art BLEU score of41.8 after training for 3.5 days on eight GPUs, a small fraction of the training costs of the best models from the literature. We show that the Transformer generalizes well to other tasks by applying it successfully to English constituency parsing both with large and limited training data.

摘要中说这篇论文介绍了一种新型的神经网络模型架构：`Transformer`，它完全基于注意力机制，在不使用循环和卷积的前提下实现序列转换任务。该模型在两个机器翻译任务中取得了表现优异的结果，训练时间缩短，且泛化性能良好。顺带一提，注意力机制是一种可以让神经网络更关注与输出结果相关的输入序列部分的技术。

我认为`Transformer`模型确实是一种非常有创新性的架构设计，训练时间更短，性能更优，谁不爱？而注意力机制也是当前深度学习领域研究的热门方向之一，它在自然语言处理、计算机视觉等领域都有着广泛应用，可以帮助神经网络更好地理解输入数据。因此，我认为` Transformer`模型和注意力机制的发展是都有着非常重要的意义的，这也是我为什么提前看这些论文的原因。

> _Q&A_

<details close>
<summary>
1. 什么是注意力机制?
</summary>
<q>
注意力机制是一种常用于神经网络模型中的技术，可以让模型在处理输入数据时更加关注与输出结果相关的部分。通俗地说，就是让模型能够将更多的注意力集中在对预测结果有贡献的输入序列部分。在序列转换任务中，注意力机制常被用于连接编码器和解码器，使得模型能够根据解码器当前的输出状态，在编码器的输入序列中寻找最相关的信息进行输出预测。相比于传统的固定权重分配方式，注意力机制能够在不同时间步根据解码器的输出状态自适应地调整权重，从而提高模型的性能表现。
</q>
</details>

<details close>
<summary>
2. 什么是 WMT?
</summary>
<q>
WMT是指机器翻译领域最具代表性的国际评测大会Workshop on Machine Translation（机器翻译研讨会），是由统计机器翻译和神经机器翻译等多个组别组成的基准测试。自2006年开始，每年都会举办一次。WMT的目标是为了提高机器翻译技术的水平，吸引更多学术界、企业参与机器翻译的研究和发展，并促进机器翻译技术应用在实际生产中。WMT评测通常会提供一些双语数据集，然后参赛者根据这些数据集训练出机器翻译模型，并使用其他测试数据集进行测试和评估，最终评选出最优秀的翻译系统。WMT评测已成为机器翻译领域公认的重要学术会议和评测竞赛之一。
</q>
</details>
  

***

_"So, the computing resources required for paper reproduction are beyond my affordability, but I'll feel regretful if I don't reproduce it. **What should I do**"<a href="/fool" title="#目移 看向阿尘和CAS.." rel="tipsy">(?)</a>_
