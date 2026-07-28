package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Value;
import prg.titech.chain.value.StringValue;

public class PrettyPrintVisitor implements ChainVisitor {
    private final StringBuilder sb = new StringBuilder();
    private final boolean doIndent;

    private boolean needCallDelimiter = false;
    private boolean needParameterDelimiter = false;

    private PrettyPrintVisitor(boolean doIndent) {
        this.doIndent = doIndent;
    }

    public static String prettyPrint(Chain chain) {
        return prettyPrint(chain, false);
    }

    public static String prettyPrint(Chain chain, boolean doIndent) {
        PrettyPrintVisitor self = new PrettyPrintVisitor(doIndent);
        chain.traverse(self);
        return self.sb.toString();
    }

    @Override
    public void visit(Chain chain) {
        needCallDelimiter = false;
    }

    @Override
    public void visit(Call call) {
        if (needCallDelimiter) {
            if (doIndent) {
                sb.append('\n');
            }
            sb.append('.');
        }
        sb.append(call.getMethodName());
        sb.append('(');
    }

    @Override
    public void endVisit(Call call) {
        sb.append(')');
        needCallDelimiter = true;
        needParameterDelimiter = false;
    }

    @Override
    public void visit(Value value) {
        if (needParameterDelimiter) {
            sb.append(", ");
        }
    }

    @Override
    public void endVisit(Value value) {
        needParameterDelimiter = true;
    }

    @Override
    public void visit(StringValue value) {
        sb.append('"');
        sb.append(value);
        sb.append('"');
    }
}
