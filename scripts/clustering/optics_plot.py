import matplotlib.pyplot as plt
import numpy as np

# This script assumes you're running from the linfa-clustering root after
# running the example so the dataset and reachability npy files are in the
# linfa-clustering root as well.

dataset = np.load("optics_clustering_dataset.npy")
reachability = np.load("optics_clustering_reachability.npy")
print(reachability)

plot1 = plt.figure(1)
plt.scatter(dataset[:, 0], dataset[:, 1])

plot2 = plt.figure(2)
x = np.arange(0, dataset.shape[0], 1)
plt.bar(x, reachability)

plt.show()