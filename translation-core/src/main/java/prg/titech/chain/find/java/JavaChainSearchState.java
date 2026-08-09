package prg.titech.chain.find.java;

import prg.titech.chain.Chain;
import prg.titech.chain.builder.ChainBuilder;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.List;

public class JavaChainSearchState {
    private final List<Chain> aggregate;
    private final Deque<ChainBuilderFrame> context;

    private static final class ChainBuilderFrame {
        ChainBuilder inner;
        int remainingMethods;

        private ChainBuilderFrame() {
            inner = Chain.builder();
            remainingMethods = 0;
        }
    }

    public JavaChainSearchState() {
        this.aggregate = new ArrayList<>();
        context = new ArrayDeque<>();
        context.add(new ChainBuilderFrame());
    }

    public ChainBuilder getCurrent() {
        return context.getLast().inner;
    }

    public void completeChain() {
        aggregate.add(getCurrent().build());
        context.getLast().inner = Chain.builder();
    }

    public void incrementRemainingMethods() {
        context.getLast().remainingMethods++;
    }

    public void decrementRemainingMethods() {
        context.getLast().remainingMethods--;
    }

    public boolean noMoreMethodsExpected() {
        return context.getLast().remainingMethods == 0;
    }

    public void descent() {
        context.addLast(new ChainBuilderFrame());
    }

    public void ascent() {
        context.removeLast();
    }

    public List<Chain> getResult() {
        return aggregate;
    }
}
