AcornDB: Suggests that it's a small seed of an idea that could grow into a mighty oak (a real database).


> Database access libraries often use two files to store the information: an index file and a
data file. The index file contains the actual index value (the key) and a pointer to the
corresponding data record in the data file. Numerous techniques can be used to
organize the index file so that it can be searched quickly and efficiently for any key:
hashing and B+ trees are popular. We have chosen to use a fixed-size hash table with
chaining for the index file. We mentioned in the description of db_openthat we create
two files: one with a suffix of .idxand one with a suffix of .dat.