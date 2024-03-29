---
layout: page
title: Publications
permalink: /publications/
---

## Papers / Eassys

Save some of my own papers and papers that I have read and think are very helpful.

### 2023

**Feb 20**

***

[Alec Radford](https://openai.com/blog/authors/alec/), [Jeffrey Wu](https://openai.com/blog/authors/jeffrey/), [Dario Amodei](https://openai.com/blog/authors/dario-amodei/), [Jack Clark](https://openai.com/blog/authors/jack-clark/), [Miles Brundage](https://openai.com/blog/authors/miles/) & [Ilya Sutskever](https://openai.com/blog/authors/ilya/)<br>
**Language Models are Unsupervised Multitask Learners**<br>
Natural language processing tasks, such as question answering, machine translation, reading comprehension, and summarization, are typically approached with supervised learning on taskspecific datasets. We demonstrate that language models begin to learn these tasks without any explicit supervision when trained on a new dataset of millions of webpages called WebText. When conditioned on a document plus questions, the answers generated by the language model reach 55 F1 on the CoQA dataset - matching or exceeding the performance of 3 out of 4 baseline systems without using the 127,000+ training examples. The capacity of the language model is essential to the success of zero-shot task transfer and increasing it improves performance in a log-linear fashion across tasks. Our largest model, GPT-2, is a 1.5B parameter Transformer that achieves state of the art results on 7 out of 8 tested language modeling datasets in a zero-shot setting but still underfits WebText. Samples from the model reflect these improvements and contain coherent paragraphs of text. These findings suggest a promising path towards building language processing systems which learn to perform tasks from their naturally occurring demonstrations.<br>
[link](https://openai.com/blog/better-language-models/) [pdf](https://d4mucfpksywv.cloudfront.net/better-language-models/language_models_are_unsupervised_multitask_learners.pdf) [code](https://github.com/openai/gpt-2)
```bibtex
@article{radford2019language,
  title={Language Models are Unsupervised Multitask Learners},
  author={Radford, Alec and Wu, Jeff and Child, Rewon and Luan, David and Amodei, Dario and Sutskever, Ilya},
  year={2019}
}
```

**Jan 20**

***

[Jeffrey Dean](https://research.google/people/jeff/), [Sanjay Ghemawat](https://research.google/people/SanjayGhemawat/)<br>
**MapReduce: Simplified Data Processing on Large Clusters**<br>
OSDI'04: Sixth Symposium on Operating System Design and Implementation, San Francisco, CA (2004), pp. 137-150<br>
[link](https://research.google/pubs/pub62/) [pdf](https://research.google/pubs/pub62.pdf) [scholar](https://scholar.google.com/scholar?lr&ie=UTF-8&oe=UTF-8&q=MapReduce%3A+Simplified+Data+Processing+on+Large+Clusters+Dean+)
```bibtex
@inproceedings{62,
title	= {MapReduce: Simplified Data Processing on Large Clusters},
author	= {Jeffrey Dean and Sanjay Ghemawat},
year	= {2004},
booktitle	= {OSDI'04: Sixth Symposium on Operating System Design and Implementation},
pages	= {137--150},
address	= {San Francisco, CA}
}
```
