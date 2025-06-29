export async function load({ cookies }) {
    return {
        has_token: cookies.get("token") !== undefined
    }
}