import numpy as num
import matplotlib.pyplot as plot

distinct_word_count = 1570894
total_words = 123113235

word_data = [
  (
     7723657,
     "the",
  ),
  (
     3938905,
     "of",
  ),
  (
     3730433,
     "and",
  ),
  (
     3417321,
     "to",
  ),
  (
     2590240,
     "a",
  ),
  (
     2562767,
     "in",
  ),
  (
     1557003,
     "is",
  ),
  (
     1425318,
     "for",
  ),
  (
     1107039,
     "that",
  ),
  (
     982392,
     "on",
  ),
  (
     919733,
     "with",
  ),
  (
     825932,
     "be",
  ),
  (
     813665,
     "it",
  ),
  (
     806040,
     "are",
  ),
  (
     790899,
     "as",
  ),
  (
     761624,
     "this",
  ),
  (
     675012,
     "you",
  ),
  (
     660339,
     "by",
  ),
  (
     609572,
     "at",
  ),
  (
     579583,
     "i",
  ),
  (
     573797,
     "from",
  ),
]

words = [tuple[1] for tuple in word_data]
counts = [tuple[0] / total_words for tuple in word_data]

top_five_probability = num.sum(counts[:5])
print(f"top five: {top_five_probability}")

x = range(len(words))

figure, chart = plot.subplots(1,1)

chart.bar(x, counts, color="#444")
chart.set_ylabel('word probability')
chart.set_xticks(x)
chart.set_xticklabels(words)

plot.show()
