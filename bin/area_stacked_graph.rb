#!/usr/bin/env ruby
# ISC License (ISC)
#
# Copyright (c) 2016, Austin Hellyer <hello@austinhellyer.me>
#
# Permission to use, copy, modify, and/or distribute this software for any
# purpose with or without fee is hereby granted, provided that the above
# copyright notice and this permission notice appear in all copies.
#
# THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
# REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
# AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
# INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
# LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
# OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
# PERFORMANCE OF THIS SOFTWARE.

# Gruff is required for generating the graph.
require 'gruff'

# JSON is required for parsing the parameters, which should be provided in JSON
# format individually.
require 'json'

if ARGV[0] == '-h' or ARGV[1] == '--help'
  puts "nanobot area-stacked-graph
Generate a stacked area graph of the provided input.

There are 3 arguments that must be given to generate the graph:
- labels: the labels, displayed at the bottom. an example of this is:
  [0, 100, 200, 300, 400, 500]
- keys: the names of the identifiers used on the graph, examples being:
  [\"Joe\", \"Bob\", \"George\"]
- values: the values for each key's points on each label, sorted precisely by
  the ordering of the keys. Example:
  [[7, 13, 21], [1, 2, 3], [7, 8, 9]]

  The first array being \"Joe\", the second \"Bob\", and the third \"George\".

The fourth argument is the path to the filepath of where to save the image, such
as /mnt/nanobot/image_temp.png

Exit codes:

- 0: success
- 1: incorrect number of arguments given
- 2: error parsing an argument from JSON
- 3: unequal number of keys and values"

  exit(0)
end

# Ensure that the number of arguments provided is 4. If it isn't, echo out the
# error and print the method of viewing the help.
if ARGV.length != 4
  puts '4 arguments required: labels keys values output_file_name. see --help'

  exit(1)
end

graph = Gruff::StackedArea.new('1600x3000')

# Set a generic title to the chart to default to.
graph.title = 'Stacked Area Chart'

# Convert the array of labels (such as [0, 1000, 2000]) into a Hash, such as:
#
# {0=>"0", 1=>"1000", 2=>"2000"}
begin
  labels = JSON.parse(ARGV[0]).map { |str| str.to_s }
rescue JSON::ParserError => error
  puts "Error converting JSON from #{error}"

  exit(2)
end

graph.labels = labels.map.with_index { |val, index| [index, val] }.to_h

begin
  keys_subbed = ARGV[1].gsub("\\u", "\\\\\\\\u").encode('utf-8')

  keys = JSON.parse(keys_subbed).map { |str| str.to_s }
  values = JSON.parse(ARGV[2])
rescue JSON::ParserError => error
  puts "Error converting JSON from #{error}"

  exit(2)
end

# Check that the number of keys provieed as equal to that of the number of
# values provided. If they are not, error-exit-code out.
for i in 0...values.length
  if values[i].length != keys.length
    puts 'There are an unequal number of keys and values'

    exit(3)
  end
end

dataset = {}

for i in 0...keys.length
  dataset[keys[i]] = values.map { |x| x[i].to_i }
end

# Loop through the number of keys (which is confirmed to be equal to the number
# of values) and insert the key and its corresponding value into the graph data.
dataset.each do |key, value|
  graph.data(key, value)
end

graph.font = '/usr/share/fonts/TTF/DejaVuSans.ttf'

# Sort the graph by the values of the keys (largest on the bottom).
graph.sort = true

# The filepath of where to save the graph should be provided as the fourth
# argument.
begin
  graph.write(ARGV[3])
rescue ArgumentError => error
  puts error.inspect
end
