package prg.titech.chain.visit;

import prg.titech.chain.Call;
import prg.titech.chain.Chain;
import prg.titech.chain.Name;
import prg.titech.chain.value.JavaExprValue;
import prg.titech.chain.value.StringValue;

import java.util.List;

public class PrettyPrintVisitor implements VoidVisitor<StringBuilder> {

    public static String prettyPrint(Visitable v) {
        PrettyPrintVisitor self = new PrettyPrintVisitor();
        StringBuilder sb = new StringBuilder();
        v.accept(self, sb);
        return sb.toString();
    }

    @Override
    public void visit(Chain chain, StringBuilder state) {
        printList(chain.getCalls(), ".", state);
    }

    @Override
    public void visit(Call call, StringBuilder state) {
        call.getMethodName().accept(this, state);
        state.append('(');
        printList(call.getParameters(), ", ", state);
        state.append(')');
    }

    @Override
    public void visit(Name name, StringBuilder state) {
        state.append(name.toString());
    }

    @Override
    public void visit(StringValue value, StringBuilder state) {
        state.append(value.toQuotedString("\""));
    }

    @Override
    public void visit(JavaExprValue value, StringBuilder state) {
        state.append(value.toString());
    }

    private <T extends Visitable> void printList(List<T> list, CharSequence delimiter, StringBuilder state) {
        boolean addDelimiter = false;
        for (T t : list) {
            if (addDelimiter) {
                state.append(delimiter);
            }
            t.accept(this, state);
            addDelimiter = true;
        }
    }
}
