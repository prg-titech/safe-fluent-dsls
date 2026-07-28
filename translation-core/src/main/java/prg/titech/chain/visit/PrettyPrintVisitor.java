package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.iter.ChainTraverser;
import prg.titech.chain.iter.context.Frame;
import prg.titech.chain.value.StringValue;

public class PrettyPrintVisitor implements ChainVisitor {
    private final StringBuilder sb = new StringBuilder();
    private final boolean doIndent;

    private PrettyPrintVisitor(boolean doIndent) {
        this.doIndent = doIndent;
    }

    public static String prettyPrint(Chain chain) {
        return prettyPrint(chain, false);
    }

    public static String prettyPrint(Chain chain, boolean doIndent) {
        PrettyPrintVisitor self = new PrettyPrintVisitor(doIndent);
        ChainTraverser traverser = new ChainTraverser(self, ChainTraverser.Strategy.ALL);
        traverser.traverse(chain);
        return self.sb.toString();
    }

    @Override
    public void visit(Frame context, Chain chain) {
        if (context.isMiddlePosition()) {
            if (doIndent) {
                sb.append('\n');
            }
            sb.append('.');
        }
    }

    @Override
    public void visit(Frame context, Call call) {
        if (context.isFirstPosition()) {
            sb.append(call.getMethodName());
            sb.append('(');
        }
        if (context.isLastPosition()) {
            sb.append(')');
        }
        if (context.isMiddlePosition()) {
            sb.append(", ");
        }
    }

    @Override
    public void visit(Frame context, StringValue value) {
        sb.append('"');
        sb.append(value);
        sb.append('"');
    }
}
