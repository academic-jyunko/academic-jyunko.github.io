---
layout: post
title: "读《Attention Is All You Need》"
author: 简律纯
mathjax: true
summary: |
  无事可做的一天，于是想起来之前自己编译的那篇"Attention Is All You Need"，我打印出来后便是再也没看过了，于是把它翻了个底朝天。在这里记下我阅读它时所做的笔记。
---

### Abstract

原文  
> The dominant sequence transduction models are based on complex recurrent or convolutional neural networks that include an encoder and a decoder. The best performing models also connect the encoder and decoder through an attention mechanism. We propose a new simple network architecture, the Transformer. based solely on attention mechanisms, dispensing with recurrence and convolutions entirely. Experiments on two machine translation tasks show these models to be superior in quality while being more parallelizable and requiring significantly less time to train. Our model achieves 28.4 BLEU on the WMT 2014 English to-German translation task, improving over the existing best results, including ensembles, by over 2 BLEU. On the WMT 2014 English-to-French translation task our model establishes a new single-model state-of-the-art BLEU score of41.8 after training for 3.5 days on eight GPUs, a small fraction of the training costs of the best models from the literature. We show that the Transformer generalizes well to other tasks by applying it successfully to English constituency parsing both with large and limited training data.

