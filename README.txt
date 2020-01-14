Algorithm's structure : 
Questions :
How to represent solution as a genome
How do we compute fitness



Generate (valid ?) random genomes
while (not over), Steps :
Selection
Cross-over
Mutation



Arbitrary factors to optimize :
Population size
Mutation probability
Number of crossover points
Elitism ? Size to choose



Constraints on validity of solution :
Every customer is on exactly on route
Every route starts and ends at the same depot
No route serves more than a preset number of customers
No route exceeds the route length limit

Minimize the total distance covered


How to :
Selection :
Choose according to fitness (and a bit of random) the pool of individuals selected for reproduction
Method names : https://en.wikipedia.org/wiki/Selection_(genetic_algorithm)

Crossover :
For each new solution to be produced, select 2 parents randomly. k-point : choose k random points, and invert 
the rest of the genomes of the parents after each point, that way you get two new children. uniform :
each bit comes from either parent with equal probabilities. You also get 2 children.

Mutation :
Apply mutation algo to each new individual

Choose a way to represent mutation/crossover that ensures only valid solutions are created ?
https://en.wikipedia.org/wiki/Crossover_(genetic_algorithm) talks about this



Other programming problems : 
Reading input and representing a problem instance, which we feed to our GA generator

Then generating an output and representing it in the required format :
Graphical and text
Every route should be of a different color



Our precise problem :
Multiple depots each with a number of vehicles and some capacities
A set of geographically dispersed customers that are to be serviced exactly once
A vehicle has to start and end at the same depot
Customers are assigned a single depot ?



Questions : 
Customers are assigned a single depot ?
Computing distance between customers ?
What's necessary service duration required for a customer ?


Time worked on this :
17h-20h

