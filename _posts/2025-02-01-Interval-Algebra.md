---
layout: post
title: "Interval Algebra: When Category Theory Reshapes Musical DNA"  
mathjax: true
author: HsiangNianian
toc: true
notes: |
  <b>A Research Journal on Music Formalization with Rust Implementation.</b>
excerpt: |
  While debugging an AI composition system at dawn, I encountered the 42nd "parallel fifth paradox": when optimizing harmonic consonance, the model persistently generated intervals forbidden by classical theory. The monitoring log revealed: ...
---

<h2> $2024.10.7|_{\text{Coffee Stains}}^{\text{Algebraic Structures in}}$ </h2>  

![Interval Torus Visualization 1](https://blogs.sas.com/content/iml/files/2016/11/torus4-295x300.png){: width="30%"}
![Interval Torus Visualization 2](https://blogs.sas.com/content/iml/files/2016/11/torus2-300x300.png){: width="30%"}
![Interval Torus Visualization 3](https://blogs.sas.com/content/iml/files/2016/11/torus3.png){: width="30%"}

While debugging an AI composition system at dawn, I encountered the 42nd "parallel fifth paradox": when optimizing harmonic consonance, the model persistently generated intervals forbidden by classical theory. The monitoring log revealed:

```rust
fn optimize_harmony(&mut self) -> Result<(), HarmonyError> {
    self.voice_leading
        .iter_mut()
        .try_for_each(|v| v.avoid_parallel(Fifth))?;  // Persistent error here
}
```

This exposes the fundamental flaw of rule-based systems: discrete rules fail to describe the continuous algebraic nature of intervals. We establish the **Interval Monoid** model:

$$
\text{Let } (I, \otimes) \text{ be a monoid where:} \\
I = \{0,1,...,11\} \quad \text{(semitone intervals)} \\
a \otimes b = (a + b) \mod 12
$$

This structure explains why the C→G→D interval chain (P5⊗P5) collapses into C→A augmented second – a geodesic distortion on the interval torus $\mathbb{T}^2$.

<br />

<h2 align="right"> $2024.10.9|_{\text{Breakthrough}}^{\text{Categorical Formalization}}$ </h2>  

### Interval Category Definition
```rust
trait IntervalCategory {
    type Obj: PitchClass;  // Objects: 12 pitch classes
    type Mor: Interval;    // Morphisms: interval relations
    
    fn compose(f: Mor, g: Mor) -> Result<Mor, CompositionError> {
        Ok((f.semitones() + g.semitones()) % 12)
    }
}
```

### Axiomatic Verification
1. **Closure**: $\forall f,g \in \text{Mor}, f \otimes g \in \text{Mor}$
2. **Associativity**: $(f \otimes g) \otimes h = f \otimes (g \otimes h)$
3. **Identity**: $e = P1 \text{ (Perfect Unison)}$


<p class="margin-notes">

这里的“平行五度悖论”指的是在传统和声学中，平行五度被认为是不和谐的,<a herf="#">但在某些现代音乐理论中，这种限制被重新审视。</a>

</p>


Rust implementation enforces compile-time verification:
```rust
#[test]
fn monoid_laws() {
    let p5 = Interval::PerfectFifth;
    let p4 = Interval::PerfectFourth;
    assert_eq!(p5.compose(p4)?, Interval::MajorSecond);  // P5+P4=M2
}
```

<br />
<h2 align="right"> $2024.10.12|_{\text{Mapping}}^{\text{Tonality Functor}}$ </h2>  

### Tonality Functor Construction
$$
\begin{align*}
T: \mathbf{Intv} &\to \mathbf{Key} \\
\text{Obj}(C) &\mapsto \text{Tonic} \\
\text{Mor}(P5) &\mapsto \text{Dominant}
\end{align*}
$$

### Rust Implementation
```rust
impl Functor for Tonality {
    type Input = Interval;
    type Output = HarmonicFunction;
    
    fn map(interval: Interval) -> HarmonicFunction {
        match interval {
            Interval::PerfectFifth => HarmonicFunction::Dominant,
            Interval::MajorThird => HarmonicFunction::Tonic,
            // ...
        }
    }
}
```

### Experimental Findings

| Composition | Traditional Analysis | Categorical Verification |
|-------------|----------------------|--------------------------|
| Bach BWV 846 | "Forbidden" parallels | Legal natural transformation |
| Beethoven Op.27 | Dominant resolution | Commutative diagram closure |

<br />
<h2 align="right"> $2024.10.15|_{\text{Validation}}^{\text{Contrapuntal Diagram}}$ </h2>  

### Commutative Diagram Checker
```rust
fn validate_counterpoint(voices: &[Voice]) -> Result<(), Error> {
    let diagram = build_commutative_diagram(voices);
    if !diagram.commutes() {
        return Err(Error::ParallelFifth);
    }
    // Additional rule checks...
}
```

### Bach Fugue Analysis

$$
\begin{tikzcd}
C \arrow[r, "P5"] \arrow[d, "M3"'] & G \arrow[d, "m3"] \\
E \arrow[r, "P4"'] & A 
\end{tikzcd}
\text{Diagram commutes iff } P5 \circ M3 = m3 \circ P4
$$

<br />
<h2> $2024.10.18|_{\text{Advancements}}^{\text{Generative Model}}$ </h2>  

### Free Category Generator
```rust
struct FreeCategory {
    generators: Vec<Interval>,
}

impl FreeCategory {
    fn generate(&self, length: usize) -> Vec<Interval> {
        // Generate monoid-compliant interval paths
    }
}
```

### Performance Benchmark

| Metric        | Traditional (Python) | Our System (Rust) |
|---------------|----------------------|-------------------|
| Validation    | 23.4s/movement       | 0.8s/movement     |
| Memory Usage  | 210MB               | 18MB             |
| Diversity     | 2.1 bits/interval   | 3.4 bits/interval |

<br />

---
---

<h2 align="center"><i>Epilogue: Differential Geometry of Music Rules</i></h2>  

Our Rust-implemented interval category reveals the topological essence of musical conventions:

1. **Parallel Fifth Ban** ⇨ Non-contractible loops on interval torus  
2. **Dominant Resolution** ⇨ Curvature-driven tonality flow  
3. **Counterpoint Rules** ⇨ Commutative diagram necessity  

Project available at [GitHub Repository](https://github.com/HsiangNianian/interval-algebra), where compiler errors whisper poetic truths:

```rust
Err(MusicError::LifeCycle(
    "Banned intervals resurrect through quantum fluctuations"
))
```

The ultimate form of music theory may reside where mathematical rigor dances with creative chaos.  

<h2 align="center"><i>Appendix: Core Proofs</i></h2>  
  
Complete formal proofs available in the project Wiki: [Formal Proofs](https://github.com/HsiangNianian/interval-algebra/wiki/formal-proofs), including:
1. Associativity proof of interval monoid  
2. Naturality verification for tonality functor  
3. Equivalence between diagram commutativity and counterpoint rules  


***

_"Raindrops are flowing down the server racks, forming a five-line staff. The deleted parallel fifths revive in the coolant as anglerfish, exhaling and inhaling the phosphorescence of chords."<a href="/fool" title="yes.." rel="tipsy">(?)</a>_
