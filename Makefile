.PHONY: start clean


PARALLEL := parallel -j2
CARGO := cargo

LOG  = $(wildcard plot/log/*.log)
DAT  = plot/fig-5-1.dat plot/fig-5-2.dat plot/fig-5-3.dat
PNG  = $(DAT:%.dat=%.png)
LOCK = .plot.lock

FOLD   := 1 2 3 4 5 6 7
MEMORY := 1 2 3 4 5 6 7


all: start

start: $(PNG)


plot/%.png: plot/%.gpi plot/%.dat
	gnuplot $< > $@

plot/fig-5-1.dat: $(LOCK)
	(seq -s ' ' 10 10 70; \
	 echo $(foreach f,$(FOLD),plot/log/ro-mid-$(f)-3.log) \
		| xargs sed -n '/compute time/s|[^0-9]||gp' - \
		| pr -7 -t \
		| datamash -W mean 1-7; \
	 echo $(foreach f,$(FOLD),plot/log/aco-mid-$(f)-3.log) \
		| xargs sed -n '/compute time/s|[^0-9]||gp' - \
		| pr -7 -t \
		| datamash -W mean 1-7; \
	 echo $(foreach f,$(FOLD),plot/log/aco-mid-$(f)-inf.log) \
		| xargs sed -n '/compute time/s|[^0-9]||gp' - \
		| pr -7 -t \
		| datamash -W mean 1-7;) \
	| datamash -W transpose | column -t > $@

plot/fig-5-2.dat: $(LOCK)
	(seq -s ' ' 10 10 70; \
	 echo $(foreach f,$(FOLD),plot/log/ro-heavy-$(f)-3.log) \
		| xargs sed -n '/compute time/s|[^0-9]||gp' - \
		| pr -7 -t \
		| datamash -W mean 1-7; \
	 echo $(foreach f,$(FOLD),plot/log/aco-heavy-$(f)-3.log) \
		| xargs sed -n '/compute time/s|[^0-9]||gp' - \
		| pr -7 -t \
		| datamash -W mean 1-7; \
	 echo $(foreach f,$(FOLD),plot/log/aco-heavy-$(f)-inf.log) \
		| xargs sed -n '/compute time/s|[^0-9]||gp' - \
		| pr -7 -t \
		| datamash -W mean 1-7;) \
	| datamash -W transpose | column -t > $@

plot/fig-5-3.dat: $(LOCK)
	(seq -s ' ' 1 7; \
	 echo $(foreach m,$(MEMORY),plot/log/aco-heavy-4-$(m).log) \
		| xargs sed -n '/compute time/s|[^0-9]||gp' - \
		| pr -7 -t \
		| datamash -W mean 1-7;) \
	| datamash -W transpose | column -t > $@

$(LOCK):
	mkdir -p plot/log/
	# for figure 5.1 and 5.2
	$(PARALLEL) $(CARGO) run -- {1} \
		exp_graph.json exp_flow_{2}.json exp_flow_reconf.json \
		{3} --config=config.{4}.json \
		'>' plot/log/{1}-{2}-{3}-{4}.log \
		::: aco ro ::: mid heavy ::: $(FOLD) ::: 3 inf
	# for figure 5.3
	$(PARALLEL) $(CARGO) run -- {1} \
		exp_graph.json exp_flow_{2}.json exp_flow_reconf.json \
		{3} --config=config.{4}.json \
		'>' plot/log/{1}-{2}-{3}-{4}.log \
		::: aco ::: heavy ::: 4 ::: $(MEMORY)
	touch $@

clean:
	$(RM) $(LOG) $(DAT) $(PNG) $(LOCK)
