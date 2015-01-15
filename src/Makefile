OUTPUT=tick2binary
OPTIMIZATIONS=-O2 -fno-exceptions
JEMALLOC=-ljemalloc -L/usr/local/Cellar/jemalloc/3.6.0/lib -I/usr/local/Cellar/jemalloc/3.6.0/include
CFLAGS=-Wall -Bstatic

all:
	clang++ $(OPTIMIZATIONS) $(CFLAGS) $(JEMALLOC) \
	*.cpp \
	-o $(OUTPUT) && strip $(OUTPUT)

clean:
	rm *.o $(OUTPUT)
