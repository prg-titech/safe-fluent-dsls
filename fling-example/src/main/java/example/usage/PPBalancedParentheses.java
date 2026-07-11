package example.usage;

import example.generated.BalancedParenthesesAST;

public class PPBalancedParentheses extends BalancedParenthesesAST.Visitor {
    StringBuilder builder = new StringBuilder();

    public static String prettyPrint(BalancedParenthesesAST.P node) {
        PPBalancedParentheses self = new PPBalancedParentheses();
        self.visit(node);
        return self.builder.toString();
    }

    @Override
    public void whileVisiting(BalancedParenthesesAST.P1 p1) {
        builder.append('(');
    }

    @Override
    public void whileVisiting(BalancedParenthesesAST.P2 p2) {
        builder.append(')');
    }
}
