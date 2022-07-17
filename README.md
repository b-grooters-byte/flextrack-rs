# flextrack-rs
Direct2D Rust model railroad flextrack demo application.

The flex track application is an extenstion of the basic bezier example. The same bezier curve is used with some additional capabilities. Recall the basic bezier formula:

$$
\begin{equation}
P(t) = \sum_{i=0}^n B_i^n(t) * P_i,t \in [0,1]
\end{equation}
$$

Model railroad flex track has effectively parallel bezier curves. These are created in the demo application by computing tangents and normals to each line segment in the primary curve and constructing offset line segments.
