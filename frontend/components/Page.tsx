type Props = {
  children: React.ReactNode;
  title?: string;
};

export function Page({ children }: Props) {
  return <div className="flex flex-col p-8 gap-2 ">{children}</div>;
}
