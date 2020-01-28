Algorithm's structure : 
Questions :
How to represent solution as a genome 
How do we compute fitness
When do we end computation



Answers :
Creating a list of ints that represent the order of customers served, 
each vehicle is separated by a zero (customers start at 1). We must 
have as much zero as we have total vehicle (minus 1). This way, we 
serve all customers exactly once for sure, and for every route to start
at the same depot, we just enforce it in the solution output AND fitness 
computation.
For the fitness, would for now say infinity if exceeds load or duration 
anywhere, else just the inverse of the distance covered (exact formula 
might have something more clever than just inverse)
For the ending, for now I do a number of generations but we can also
think about combining it with a minimum progress over X generations



Generate random genomes
while (not over), Steps :
Selection
Cross-over
Mutation



Arbitrary factors and choices to optimize :
Population size
Mutation probability
Number of crossover points
Elitism ? Size to choose
Pool size during selection (Environement pressure ?)
Number of generations
__________
Selection, mutation, cross-over, fitness functions
End of computation



Constraints on validity of solution :
Every customer is on exactly on route
Every route starts and ends at the same depot
No route serves more than a preset number of customer demands
No route exceeds the route length limit

Minimize the total distance covered



How to :
Selection :
Choose according to fitness (and a bit of random) the pool of individuals selected for reproduction
Method names : https://en.wikipedia.org/wiki/Selection_(genetic_algorithm)
Roulette wheel : Based probability of being chosen on fitness (example, fitness over total fitness)
Rank selection : Same but base probability on rank only. Here we have probability = rank/total_ranks
with ranks starting at zero

Crossover :
For each new solution to be produced, select 2 parents randomly. k-point : choose k random points, and invert 
the rest of the genomes of the parents after each point, that way you get two new children. uniform :
each bit comes from either parent with equal probabilities. You also get 2 children.

Mutation :
Apply mutation algo to each new individual. Here we give each object a probability of swapping with a uniformly
chosen random one

Choose a way to represent mutation/crossover that ensures only valid solutions are created ?
https://en.wikipedia.org/wiki/Crossover_(genetic_algorithm) talks about this



Other programming problems : 
Reading input and representing a problem instance, which we feed to our GA generator (DONE)

Then generating an output and representing it in the required format :
Graphical and text
Every route should be of a different color



Our precise problem :
Multiple depots each with a number of vehicles and some capacities
A set of geographically dispersed customers that are to be serviced exactly once
A vehicle has to start and end at the same depot
Customers are assigned a single depot ?



Problems :
Converges too quickly, doesn't explore enough possibilities ?
What is duration ? And customer duration requirement ?


TODO : 
What takes time is computing the crossovers, so we have to make those faster



Test runs on 23 :
Parameters : Tournament 50, 200, 500, mutation 0.2 to 0.6
Then : Fractions of mutation
Results :
R, 0.2 		17513, 31m13s
50T, 0.2	11478, 28m54s
200T, 0.2 	11486, 30m37s
500T, 0.2 	11058, 29m39s
1000T, 0.2 	11739, 28m48s
50T, 0.3
200T, 0.3 
500T, 0.3 	11735, 28m21s
50T, 0.4
200T, 0.4 
500T, 0.4 	10619, 29m47s
50T, 0.5
200T, 0.5 
500T, 0.5 	11141, 30m07s
50T, 0.6
200T, 0.6 
500T, 0.6 	12030, 30m02s

1 de chaque : 10340