function replaceDomain(url: string) {
  return url.replace("http://localhost:3000", "http://127.0.0.1:3001");
}

export async function GET(request: Request) {
  console.log(replaceDomain(request.url));
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "GET",
  });
  return res;
}

export async function POST(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "POST",
    body: request.body,
  });
  return res;
}

export async function PUT(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "PUT",
    body: request.body,
  });
  return res;
}

export async function DELETE(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "DELETE",
  });
  return res;
}

export async function PATCH(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "PATCH",
    body: request.body,
  });
  return res;
}

export async function OPTIONS(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "OPTIONS",
  });
  return res;
}

export async function HEAD(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "HEAD",
  });
  return res;
}

export async function CONNECT(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "CONNECT",
  });
  return res;
}

export async function TRACE(request: Request) {
  const res = await fetch(replaceDomain(request.url), {
    headers: request.headers,
    method: "TRACE",
  });
  return res;
}
