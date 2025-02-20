

type Props = {
  title: string;
  children?: React.ReactNode;
};

export function SectionTitle({ children }: Props) {
  return (
    <div className="flex gap-4">
      <h1 className="text-2xl">Geräte</h1>
      {children}
    </div>
  );
}
