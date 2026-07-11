package org.example;

import example.generated.BalancedParentheses;
import example.usage.PPBalancedParentheses;

import static example.generated.BalancedParenthesesAST.P;

public class Main {

    public static void main(String[] args) {
        P paren = BalancedParentheses.a().a().a().a().c().a().c().c().c().c().$();

        System.out.printf("%s%n", PPBalancedParentheses.prettyPrint(paren));
    }
}