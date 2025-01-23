type Props = {
  children: React.ReactNode;
  title: string;
};

export function Page({ children, title }: Props) {
  return (
    <div className="flex flex-col p-8 gap-2">
      <h1 className="text-2xl font-bold">{title}</h1>
      {children}
    </div>
  );
}
