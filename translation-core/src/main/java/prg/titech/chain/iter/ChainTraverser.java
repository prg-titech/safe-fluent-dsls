package prg.titech.chain.iter;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Value;
import prg.titech.chain.iter.context.Context;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.visit.ChainVisitor;
import prg.titech.chain.visit.Visitable;

import java.util.function.Predicate;

public class ChainTraverser {
    private final Context context = new Context();
    private final ChainVisitor visitor;
    private final Predicate<Frame> strategy;

    public enum Strategy {
        PREORDER,
        POSTORDER,
        ALL
    }

    public ChainTraverser(ChainVisitor visitor, Strategy strategy) {
        this.visitor = visitor;
        this.strategy = switch (strategy) {
            case PREORDER -> Frame::isFirstPosition;
            case POSTORDER -> Frame::isLastPosition;
            case ALL -> f -> true;
        };
    }

    public void traverse(Chain chain) {
        context.descent();
        for (Call call : chain.getCalls()) {
            accept(chain);
            call.accept(this);
        }
        context.setLastPosition();
        accept(chain);
        context.ascent();
    }

    public void traverse(Call call) {
        context.setCurrentMethod(call.getMethodName());
        context.descent();
        for (Value v : call.getParameters()) {
            accept(call);
            v.accept(this);
        }
        context.setLastPosition();
        accept(call);
        context.ascent();
        context.setCurrentMethod(null);
    }

    public void traverse(Value value) {
        context.descent();
        context.setLastPosition();
        accept(value);
        context.ascent();
    }

    private void accept(Visitable v) {
        Frame context = this.context.getCurrentFrame();
        if (strategy.test(context)) {
            v.accept(context, visitor);
        }
        context.incrementPosition();
    }

}
