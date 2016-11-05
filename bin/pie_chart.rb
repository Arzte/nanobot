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

require 'gruff'

keys = ARGV[0].split(',')
values = ARGV[1].split(',')

# Ensure that the number of keys (names) is equal to the number of values. i.e.,
# ensure that this is what happens:
#
# - a: 5
# - b: 11
# - c: 13
#
# and not something like:
#
# - a: 5
# - b: 12
# - c:
# - d:
if keys.length != values.length
  puts 'There are an unequal number of keys and values'

  exit(1)
end

graph = Gruff::Pie.new

graph.title = 'Pie Chart'

# Add each key-value pair to the graph's data.
for i in 0...keys.length
  graph.data keys[i], values[i].to_i
end

graph.font = '/usr/share/fonts/TTF/DejaVuSans.ttf'

# Finally, write the output image to a filepath.
graph.write './img_temp.png'

exit(0)
