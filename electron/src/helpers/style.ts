import { useMemo } from "react";

type Variable = {
  [key: string]: string;
};

type Variables = {
  [key: string]: Variable;
};

type VariableCombinations<V extends Variables> = ({
  [K in keyof V]: { [key in keyof V[K]]: string };
} & {
  className: string;
})[];

type DefaultVariables<V extends Variables> = {
  [K in keyof V]: keyof V[K];
};

export type Props<V extends Variables> = {
  base?: string;
  variables?: V;
  variableCombinations?: VariableCombinations<V>;
  defaultVariables?: DefaultVariables<V>;
};

type VariablesInput<V extends Variables> = {
  [K in keyof V]: keyof V[K];
} & {
  className?: string;
};

export function classNameBuilder<V extends Variables>(
  props: Props<V>,
): (variables: VariablesInput<V>) => string {
  return (variables) => {
    const classNames: string[] = [];

    // add base
    if (props.base) {
      classNames.push(props.base);
    }

    // iterate props.variables keys
    // if variable is undefined, use default
    for (const variableKey of Object.keys(props.variables || {})) {
      // check if variableKey is valid
      if (!variables[variableKey]) {
        continue;
      }
      // get valueKey
      const valueKey = variables[variableKey] as string | number;
      // get variable value
      const value = (props.variables?.[variableKey]?.[valueKey] ||
        props.defaultVariables?.[variableKey]) as string;
      if (value) {
        classNames.push(value);
      }
    }

    // iterate variable combinations
    for (const combination of props.variableCombinations || []) {
      // check if all variable keys are valid
      if (
        Object.keys(combination).every(
          (key) => variables[key] === combination[key],
        )
      ) {
        classNames.push(combination.className);
      }
    }

    // add custom className
    if (variables.className) {
      classNames.push(variables.className);
    }

    console.log("classNames", classNames);

    return classNames.join(" ");
  };
}

export function useClassNameBuilder<V extends Variables>(
  props: Props<V>,
): (variables: VariablesInput<V>) => string {
  const classNameBuilderMemo = useMemo(() => classNameBuilder(props), [props]);
  return classNameBuilderMemo;
}
