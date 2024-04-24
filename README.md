# flextrack-rs
Direct2D Rust model railroad flextrack demo application.

The flex track application is an extenstion of the basic Bézier example. The same Bézier curve is used with some additional capabilities. Recall the basic Bézier formula:

$$
\begin{equation}
P(t) = \sum_{i=0}^n B_i^n(t) * P_i,t \in [0,1]
\end{equation}
$$

Model railroad flex track can be rendered with parallel bezier curves. These are created in the demo application by computing tangents and normals to each line segment in the primary curve and constructing offset line segments. The offset line segments make up the "rails" of the flex track. The ties are rendered as rectangles at specific offsets along the primary curve.

## Tangents and Normals

The tangent to a curve at a point is the derivative of the curve at that point. The normal to a curve at a point is the vector perpendicular to the tangent at that point. The tangent and normal vectors are used to construct the offset line segments. This is done by computing the tangent points for the curve at the current resolution. The tangent points are then used to construct the offset line segments by computing the normal vectors and scaling them by the desired offset distance.

