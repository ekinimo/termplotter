* Regrets
This project started to test my parser combinator and ballooned beyond recognition. NIH syndrome plagues the codebase 
A long hiatus also didnt help with the consistency of the codebase. I commited to a design back in the day that im not really happy about (though if done consistently would have created great flexibility) 

* Improvements
- Evaluation should be rethought and reimplemented. Vars should be replaced with the proper generic as in the original vision 
- Evaluation structs suffer from uniform partitioning of the x values. We should have an adaptive method. We should take the derivative of the expression into account!
- We need proper interpolation. Currently we just draw a line between subsequent graph points
- 3d parametric plot is missing
- implicit plots are missing
- We probably need something better than f64. Interval arithmetic is the way to go
- API is not flexible enough. We should let user specify styling/view point
